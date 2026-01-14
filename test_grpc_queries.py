#!/usr/bin/env python3
"""
Python program to test gRPC SearchMemories endpoint with randomized queries.
Uses grpcurl under the hood to make the calls.
"""

import subprocess
import json
import random
import time
import argparse
from typing import Optional, Dict, List
from dataclasses import dataclass


@dataclass
class GrpcConfig:
    host: str = "localhost"
    port: int = 5051
    service: str = "memory_v1.MemoryService/SearchMemories"

    @property
    def endpoint(self) -> str:
        return f"{self.host}:{self.port}"


class QueryGenerator:
    """Generates randomized queries for testing."""

    QUERY_SEEDS = {
        "food": ["pasta", "pizza", "sushi", "tacos", "burger", "salad", "soup", "rice"],
        "adjectives": [
            "perfect",
            "delicious",
            "amazing",
            "tasty",
            "spicy",
            "sweet",
            "crispy",
        ],
        "cooking": [
            "recipe",
            "cooking",
            "baking",
            "grilling",
            "frying",
            "steaming",
            "baking",
        ],
        "topics": ["memory", "learning", "AI", "coding", "python", "rust", "databases"],
        "actions": ["ate", "made", "learned", "discussed", "studied", "implemented"],
    }

    def __init__(self, seed: Optional[int] = None):
        if seed is not None:
            random.seed(seed)

    def random_simple_query(self) -> str:
        """Generate a simple 1-2 word query."""
        category = random.choice(list(self.QUERY_SEEDS.keys()))
        word = random.choice(self.QUERY_SEEDS[category])
        return word

    def random_phrase_query(self) -> str:
        """Generate a multi-word query like 'perfect pasta'."""
        adjective = random.choice(self.QUERY_SEEDS["adjectives"])
        category = random.choice(list(self.QUERY_SEEDS.keys()))
        if category == "adjectives":
            category = "food"
        noun = random.choice(self.QUERY_SEEDS[category])
        return f"{adjective} {noun}"

    def random_sentence_query(self) -> str:
        """Generate a sentence-like query."""
        action = random.choice(self.QUERY_SEEDS["actions"])
        adjective = random.choice(self.QUERY_SEEDS["adjectives"])
        category = random.choice(list(self.QUERY_SEEDS.keys()))
        if category == "adjectives":
            category = "food"
        noun = random.choice(self.QUERY_SEEDS[category])
        return f"{action} {adjective} {noun}"

    def random_query(self, query_type: str = "any") -> str:
        """Generate a random query of specified type."""
        query_types = {
            "simple": self.random_simple_query,
            "phrase": self.random_phrase_query,
            "sentence": self.random_sentence_query,
        }

        if query_type == "any":
            query_type = random.choice(["simple", "phrase", "sentence"])

        generator = query_types.get(query_type, self.random_simple_query)
        return generator()


class GrpcClient:
    """Client for interacting with gRPC service via grpcurl."""

    def __init__(self, config: GrpcConfig):
        self.config = config

    def search_memories(
        self,
        query: str,
        user_id: Optional[str] = None,
        agent_id: Optional[str] = None,
        run_id: Optional[str] = None,
    ) -> Dict:
        """
        Call SearchMemories gRPC endpoint with the given query and context.
        Returns parsed JSON response.
        """
        # Build the request JSON
        request = {"query": query}

        context = {}
        if user_id:
            context["user_id"] = user_id
        if agent_id:
            context["agent_id"] = agent_id
        if run_id:
            context["run_id"] = run_id

        if context:
            request["context"] = context

        # Build grpcurl command
        cmd = [
            "grpcurl",
            "-plaintext",
            "-d",
            json.dumps(request),
            self.config.endpoint,
            self.config.service,
        ]

        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                text=True,
                timeout=10,
            )

            if result.returncode != 0:
                return {
                    "success": False,
                    "error": result.stderr,
                    "query": query,
                }

            # Parse the response
            response = json.loads(result.stdout)
            return {
                "success": True,
                "query": query,
                "response": response,
                "memory_count": len(response.get("memories", [])),
            }

        except subprocess.TimeoutExpired:
            return {
                "success": False,
                "error": "Request timed out",
                "query": query,
            }
        except json.JSONDecodeError as e:
            return {
                "success": False,
                "error": f"Failed to parse response: {e}",
                "query": query,
                "raw_output": result.stdout,
            }
        except Exception as e:
            return {
                "success": False,
                "error": str(e),
                "query": query,
            }


def format_result(result: Dict) -> str:
    """Format a single result for display."""
    if not result["success"]:
        return f"❌ Query: '{result['query']}' | Error: {result['error']}"

    memory_count = result.get("memory_count", 0)
    return f"✓ Query: '{result['query']}' | Found {memory_count} memories"


def main():
    parser = argparse.ArgumentParser(
        description="Test gRPC SearchMemories endpoint with randomized queries"
    )
    parser.add_argument(
        "-n",
        "--num-queries",
        type=int,
        default=1,
        help="Number of random queries to send (default: 5)",
    )
    parser.add_argument(
        "-u",
        "--user-id",
        type=str,
        default="harry",
        help="User ID for context (default: harry)",
    )
    parser.add_argument(
        "-a",
        "--agent-id",
        type=str,
        help="Agent ID for context (optional)",
    )
    parser.add_argument(
        "-r",
        "--run-id",
        type=str,
        help="Run ID for context (optional)",
    )
    parser.add_argument(
        "-t",
        "--query-type",
        choices=["any", "simple", "phrase", "sentence"],
        default="any",
        help="Type of queries to generate (default: any)",
    )
    parser.add_argument(
        "-d",
        "--delay",
        type=float,
        default=0.1,
        help="Delay between requests in seconds (default: 0.1)",
    )
    parser.add_argument(
        "-s",
        "--seed",
        type=int,
        help="Random seed for reproducible queries (optional)",
    )
    parser.add_argument(
        "--host",
        default="localhost",
        help="gRPC server host (default: localhost)",
    )
    parser.add_argument(
        "--port",
        type=int,
        default=5051,
        help="gRPC server port (default: 5051)",
    )
    parser.add_argument(
        "-v",
        "--verbose",
        action="store_true",
        help="Print detailed response information",
    )

    args = parser.parse_args()

    # Initialize client and query generator
    config = GrpcConfig(host=args.host, port=args.port)
    client = GrpcClient(config)
    generator = QueryGenerator(seed=args.seed)

    print(f"Testing gRPC endpoint at {config.endpoint}")
    print(f"Service: {config.service}")
    print(f"Sending {args.num_queries} queries with user_id='{args.user_id}'")
    print(f"Query type: {args.query_type}")
    if args.seed is not None:
        print(f"Random seed: {args.seed}")
    print("-" * 80)

    results: List[Dict] = []
    successful = 0
    failed = 0

    for i in range(args.num_queries):
        # Generate random query
        query = generator.random_query(query_type=args.query_type)

        # Make the request
        result = client.search_memories(
            query=query,
            user_id=args.user_id,
            agent_id=args.agent_id,
            run_id=args.run_id,
        )

        results.append(result)

        # Print result
        print(f"[{i + 1}/{args.num_queries}] {format_result(result)}")

        if result["success"]:
            successful += 1
            if args.verbose and result.get("response"):
                memories = result["response"].get("memories", [])
                for memory in memories[:2]:  # Show first 2 memories
                    summary = memory.get("content", {}).get("summary", "N/A")
                    print(f"    └─ {summary[:70]}")
        else:
            failed += 1

        # Add delay between requests
        if i < args.num_queries - 1:
            time.sleep(args.delay)

    print("-" * 80)
    print(f"Results: {successful} successful, {failed} failed")
    if args.seed is not None:
        print(f"To reproduce these results, use: --seed {args.seed}")


if __name__ == "__main__":
    main()
