"""
Fetch 100K random Wikipedia pages (title + description) and save to JSON.

Uses the Wikipedia API's random endpoint in batches of 500 (max allowed).
Output: examples/wikipedia_100k.json

Usage:
    pip install requests
    python examples/fetch_wikipedia.py
"""

import json
import time
import requests

API_URL = "https://en.wikipedia.org/w/api.php"
HEADERS = {"User-Agent": "kvdb-demo/1.0 (vector database demo; educational project)"}
TARGET = 100_000
BATCH_SIZE = 500  # Wikipedia API max for list=random
OUTPUT_FILE = "examples/wikipedia_100k.json"


def fetch_random_titles(count: int) -> list[str]:
    """Fetch a batch of random page titles."""
    params = {
        "action": "query",
        "format": "json",
        "list": "random",
        "rnnamespace": 0,  # Main namespace only (articles)
        "rnlimit": count,
    }
    resp = requests.get(API_URL, params=params, headers=HEADERS, timeout=30)
    resp.raise_for_status()
    data = resp.json()
    return [page["title"] for page in data["query"]["random"]]


def fetch_descriptions(titles: list[str]) -> dict[str, str]:
    """Fetch short descriptions for a batch of titles (max 50 per request)."""
    results = {}
    # API allows max 50 titles per query for prop=description
    for i in range(0, len(titles), 50):
        batch = titles[i:i + 50]
        params = {
            "action": "query",
            "format": "json",
            "titles": "|".join(batch),
            "prop": "description",
        }
        resp = requests.get(API_URL, params=params, headers=HEADERS, timeout=30)
        resp.raise_for_status()
        pages = resp.json().get("query", {}).get("pages", {})
        for page in pages.values():
            title = page.get("title", "")
            desc = page.get("description", "")
            if title and desc:
                results[title] = desc
    return results


def main():
    all_pages = {}
    batch_num = 0

    print(f"Fetching {TARGET} random Wikipedia pages...")
    print(f"Output: {OUTPUT_FILE}\n")

    while len(all_pages) < TARGET:
        batch_num += 1
        remaining = TARGET - len(all_pages)
        fetch_count = min(BATCH_SIZE, remaining)

        try:
            # Step 1: Get random titles
            titles = fetch_random_titles(fetch_count)

            # Step 2: Get descriptions for those titles
            described = fetch_descriptions(titles)
            all_pages.update(described)

            collected = len(all_pages)
            print(f"  Batch {batch_num}: +{len(described)} pages "
                  f"({collected}/{TARGET}, {collected * 100 / TARGET:.1f}%)")

            # Rate limit: be polite to Wikipedia API
            time.sleep(0.5)

        except requests.RequestException as e:
            print(f"  Batch {batch_num}: Error - {e}, retrying in 5s...")
            time.sleep(5)
            continue

    # Trim to exact target if we overshot
    pages_list = [
        {"title": title, "description": desc}
        for title, desc in list(all_pages.items())[:TARGET]
    ]

    # Save to JSON
    with open(OUTPUT_FILE, "w", encoding="utf-8") as f:
        json.dump(pages_list, f, ensure_ascii=False, indent=2)

    print(f"\nDone! Saved {len(pages_list)} pages to {OUTPUT_FILE}")
    file_size_mb = len(json.dumps(pages_list, ensure_ascii=False)) / 1_048_576
    print(f"File size: ~{file_size_mb:.1f} MB")

    # Show some samples
    print("\nSample entries:")
    for page in pages_list[:5]:
        print(f"  [{page['title']}] {page['description']}")


if __name__ == "__main__":
    main()