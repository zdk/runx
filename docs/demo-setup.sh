#!/bin/sh
# Creates a reproducible demo repo for VHS recordings
# Usage: bash docs/demo-setup.sh

set -e

DEMO_DIR="/tmp/lowfat-demo"

rm -rf "$DEMO_DIR"
mkdir -p "$DEMO_DIR"
cd "$DEMO_DIR"
git init -q
git config user.email "demo@example.com"
git config user.name "demo"

# --- Initial committed state ---

cat > app.py << 'PYEOF'
from flask import Flask, jsonify, request

app = Flask(__name__)

# In-memory store
items = []

@app.route("/items", methods=["GET"])
def list_items():
    return jsonify(items)

@app.route("/items", methods=["POST"])
def create_item():
    data = request.get_json()
    item = {"id": len(items) + 1, "name": data["name"], "done": False}
    items.append(item)
    return jsonify(item), 201

@app.route("/items/<int:item_id>", methods=["PUT"])
def update_item(item_id):
    for item in items:
        if item["id"] == item_id:
            item["name"] = request.get_json().get("name", item["name"])
            item["done"] = request.get_json().get("done", item["done"])
            return jsonify(item)
    return jsonify({"error": "not found"}), 404

if __name__ == "__main__":
    app.run(debug=True, port=5000)
PYEOF

cat > config.yaml << 'YAMLEOF'
app:
  name: todo-api
  port: 5000
  debug: true

database:
  url: sqlite:///todo.db
  pool_size: 5
YAMLEOF

cat > tests.py << 'PYEOF'
import unittest
from app import app

class TestItems(unittest.TestCase):
    def setUp(self):
        self.client = app.test_client()

    def test_list_empty(self):
        resp = self.client.get("/items")
        self.assertEqual(resp.status_code, 200)
        self.assertEqual(resp.get_json(), [])

    def test_create_item(self):
        resp = self.client.post("/items", json={"name": "buy milk"})
        self.assertEqual(resp.status_code, 201)
        self.assertIn("id", resp.get_json())

if __name__ == "__main__":
    unittest.main()
PYEOF

git add app.py
git commit -q -m "feat: initial todo API with CRUD endpoints"

git add config.yaml
git commit -q -m "chore: add app config with sqlite defaults"

git add tests.py
git commit -q -m "test: add unit tests for list and create"

# --- Second round of committed changes ---

cat > requirements.txt << 'EOF'
flask==3.0.0
gunicorn==21.2.0
python-dotenv==1.0.0
EOF
git add requirements.txt
git commit -q -m "chore: add requirements.txt with flask and gunicorn"

cat > Dockerfile << 'EOF'
FROM python:3.12-slim
WORKDIR /app
COPY requirements.txt .
RUN pip install -r requirements.txt
COPY . .
CMD ["gunicorn", "app:app", "-b", "0.0.0.0:8080"]
EOF
git add Dockerfile
git commit -q -m "feat: add Dockerfile for container deployment"

mkdir -p .github/workflows
cat > .github/workflows/ci.yml << 'EOF'
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: actions/setup-python@v5
        with:
          python-version: "3.12"
      - run: pip install -r requirements.txt
      - run: python -m pytest tests.py
EOF
git add -A
git commit -q -m "ci: add GitHub Actions workflow for tests"

cat > README.md << 'EOF'
# todo-api

A simple REST API for managing todo items.

## Run locally

    pip install -r requirements.txt
    python app.py

## API

- GET /items - list all items
- POST /items - create an item
- PUT /items/:id - update an item
EOF
git add README.md
git commit -q -m "docs: add README with API docs"

# Add middleware and models for more file diversity
cat > middleware.py << 'PYEOF'
import time
import logging
from functools import wraps
from flask import request, jsonify

logger = logging.getLogger(__name__)

def require_json(f):
    @wraps(f)
    def decorated(*args, **kwargs):
        if not request.is_json:
            return jsonify({"error": "Content-Type must be application/json"}), 415
        return f(*args, **kwargs)
    return decorated

def log_request(f):
    @wraps(f)
    def decorated(*args, **kwargs):
        start = time.time()
        response = f(*args, **kwargs)
        duration = time.time() - start
        logger.info("%s %s %.3fs %s", request.method, request.path, duration, response.status_code if hasattr(response, 'status_code') else '')
        return response
    return decorated

def validate_item(data):
    errors = []
    if not data.get("name"):
        errors.append("name is required")
    if len(data.get("name", "")) > 200:
        errors.append("name must be under 200 characters")
    if "priority" in data and data["priority"] not in ("low", "medium", "high"):
        errors.append("priority must be low, medium, or high")
    return errors
PYEOF
git add middleware.py
git commit -q -m "feat: add request middleware and validation helpers"

cat > models.py << 'PYEOF'
from dataclasses import dataclass, field, asdict
from datetime import datetime
from typing import Optional
import uuid

@dataclass
class Item:
    name: str
    id: str = field(default_factory=lambda: str(uuid.uuid4())[:8])
    done: bool = False
    priority: str = "medium"
    created_at: str = field(default_factory=lambda: datetime.utcnow().isoformat())
    updated_at: Optional[str] = None
    tags: list = field(default_factory=list)

    def to_dict(self):
        return asdict(self)

    def update(self, data: dict):
        for key in ("name", "done", "priority", "tags"):
            if key in data:
                setattr(self, key, data[key])
        self.updated_at = datetime.utcnow().isoformat()

class ItemStore:
    def __init__(self):
        self._items: dict[str, Item] = {}

    def add(self, item: Item) -> Item:
        self._items[item.id] = item
        return item

    def get(self, item_id: str) -> Optional[Item]:
        return self._items.get(item_id)

    def list_all(self, status=None, priority=None):
        items = list(self._items.values())
        if status == "done":
            items = [i for i in items if i.done]
        elif status == "pending":
            items = [i for i in items if not i.done]
        if priority:
            items = [i for i in items if i.priority == priority]
        return items

    def delete(self, item_id: str) -> bool:
        return self._items.pop(item_id, None) is not None

    def count(self):
        return len(self._items)
PYEOF
git add models.py
git commit -q -m "feat: add Item dataclass and ItemStore"

# --- Uncommitted changes (produces a rich diff) ---

cat > app.py << 'PYEOF'
from flask import Flask, jsonify, request
from datetime import datetime
import logging

app = Flask(__name__)
logging.basicConfig(level=logging.INFO)
logger = logging.getLogger(__name__)

# In-memory store
items = []

@app.route("/health", methods=["GET"])
def health_check():
    return jsonify({"status": "ok", "timestamp": datetime.utcnow().isoformat()})

@app.route("/items", methods=["GET"])
def list_items():
    status = request.args.get("status")
    if status == "done":
        return jsonify([i for i in items if i["done"]])
    elif status == "pending":
        return jsonify([i for i in items if not i["done"]])
    return jsonify(items)

@app.route("/items", methods=["POST"])
def create_item():
    data = request.get_json()
    if not data or "name" not in data:
        return jsonify({"error": "name is required"}), 400
    item = {
        "id": len(items) + 1,
        "name": data["name"],
        "done": False,
        "created_at": datetime.utcnow().isoformat(),
    }
    items.append(item)
    logger.info("created item %d: %s", item["id"], item["name"])
    return jsonify(item), 201

@app.route("/items/<int:item_id>", methods=["PUT"])
def update_item(item_id):
    for item in items:
        if item["id"] == item_id:
            item["name"] = request.get_json().get("name", item["name"])
            item["done"] = request.get_json().get("done", item["done"])
            logger.info("updated item %d", item_id)
            return jsonify(item)
    return jsonify({"error": "not found"}), 404

@app.route("/items/<int:item_id>", methods=["DELETE"])
def delete_item(item_id):
    for i, item in enumerate(items):
        if item["id"] == item_id:
            items.pop(i)
            logger.info("deleted item %d", item_id)
            return "", 204
    return jsonify({"error": "not found"}), 404

if __name__ == "__main__":
    app.run(debug=True, port=8080)
PYEOF

cat > config.yaml << 'YAMLEOF'
app:
  name: todo-api
  port: 8080
  debug: false
  log_level: info

database:
  url: postgresql://localhost:5432/todo
  pool_size: 10
  timeout: 30

redis:
  url: redis://localhost:6379
  ttl: 300
YAMLEOF

cat > tests.py << 'PYEOF'
import unittest
from app import app

class TestItems(unittest.TestCase):
    def setUp(self):
        self.client = app.test_client()
        # Clear items between tests
        import app as app_mod
        app_mod.items.clear()

    def test_list_empty(self):
        resp = self.client.get("/items")
        self.assertEqual(resp.status_code, 200)
        self.assertEqual(resp.get_json(), [])

    def test_create_item(self):
        resp = self.client.post("/items", json={"name": "buy milk"})
        self.assertEqual(resp.status_code, 201)
        data = resp.get_json()
        self.assertIn("id", data)
        self.assertIn("created_at", data)

    def test_create_item_missing_name(self):
        resp = self.client.post("/items", json={})
        self.assertEqual(resp.status_code, 400)

    def test_filter_by_status(self):
        self.client.post("/items", json={"name": "task 1"})
        resp = self.client.get("/items?status=pending")
        self.assertEqual(len(resp.get_json()), 1)

    def test_delete_item(self):
        self.client.post("/items", json={"name": "to delete"})
        resp = self.client.delete("/items/1")
        self.assertEqual(resp.status_code, 204)

    def test_health_check(self):
        resp = self.client.get("/health")
        self.assertEqual(resp.status_code, 200)
        self.assertIn("status", resp.get_json())

if __name__ == "__main__":
    unittest.main()
PYEOF

# Also modify middleware.py and models.py (uncommitted)
cat > middleware.py << 'PYEOF'
import time
import logging
from functools import wraps
from flask import request, jsonify, g
from datetime import datetime

logger = logging.getLogger(__name__)

def require_json(f):
    @wraps(f)
    def decorated(*args, **kwargs):
        if not request.is_json:
            return jsonify({"error": "Content-Type must be application/json"}), 415
        return f(*args, **kwargs)
    return decorated

def log_request(f):
    @wraps(f)
    def decorated(*args, **kwargs):
        g.request_start = time.time()
        response = f(*args, **kwargs)
        duration = time.time() - g.request_start
        logger.info(
            "%s %s %d %.3fms",
            request.method,
            request.path,
            response.status_code if hasattr(response, 'status_code') else 0,
            duration * 1000,
        )
        return response
    return decorated

def rate_limit(max_requests=100, window=60):
    """Simple in-memory rate limiter per IP."""
    from collections import defaultdict
    counters = defaultdict(list)
    def decorator(f):
        @wraps(f)
        def decorated(*args, **kwargs):
            ip = request.remote_addr
            now = time.time()
            counters[ip] = [t for t in counters[ip] if now - t < window]
            if len(counters[ip]) >= max_requests:
                return jsonify({"error": "rate limit exceeded"}), 429
            counters[ip].append(now)
            return f(*args, **kwargs)
        return decorated
    return decorator

def validate_item(data):
    errors = []
    if not data:
        return ["request body is required"]
    if not data.get("name"):
        errors.append("name is required")
    if len(data.get("name", "")) > 200:
        errors.append("name must be under 200 characters")
    if "priority" in data and data["priority"] not in ("low", "medium", "high", "urgent"):
        errors.append("priority must be low, medium, high, or urgent")
    if "tags" in data:
        if not isinstance(data["tags"], list):
            errors.append("tags must be a list")
        elif len(data["tags"]) > 10:
            errors.append("maximum 10 tags allowed")
    if "due_date" in data:
        try:
            datetime.fromisoformat(data["due_date"])
        except ValueError:
            errors.append("due_date must be ISO 8601 format")
    return errors

def handle_errors(app):
    @app.errorhandler(404)
    def not_found(e):
        return jsonify({"error": "not found"}), 404

    @app.errorhandler(500)
    def server_error(e):
        logger.exception("internal server error")
        return jsonify({"error": "internal server error"}), 500
PYEOF

cat > models.py << 'PYEOF'
from dataclasses import dataclass, field, asdict
from datetime import datetime
from typing import Optional
import uuid

@dataclass
class Item:
    name: str
    id: str = field(default_factory=lambda: str(uuid.uuid4())[:8])
    done: bool = False
    priority: str = "medium"
    created_at: str = field(default_factory=lambda: datetime.utcnow().isoformat())
    updated_at: Optional[str] = None
    due_date: Optional[str] = None
    tags: list = field(default_factory=list)

    def to_dict(self):
        d = asdict(self)
        d["overdue"] = self.is_overdue()
        return d

    def update(self, data: dict):
        for key in ("name", "done", "priority", "tags", "due_date"):
            if key in data:
                setattr(self, key, data[key])
        self.updated_at = datetime.utcnow().isoformat()

    def is_overdue(self) -> bool:
        if not self.due_date or self.done:
            return False
        try:
            return datetime.fromisoformat(self.due_date) < datetime.utcnow()
        except ValueError:
            return False

class ItemStore:
    def __init__(self):
        self._items: dict[str, Item] = {}

    def add(self, item: Item) -> Item:
        self._items[item.id] = item
        return item

    def get(self, item_id: str) -> Optional[Item]:
        return self._items.get(item_id)

    def list_all(self, status=None, priority=None, tag=None):
        items = list(self._items.values())
        if status == "done":
            items = [i for i in items if i.done]
        elif status == "pending":
            items = [i for i in items if not i.done]
        elif status == "overdue":
            items = [i for i in items if i.is_overdue()]
        if priority:
            items = [i for i in items if i.priority == priority]
        if tag:
            items = [i for i in items if tag in i.tags]
        return sorted(items, key=lambda i: i.created_at, reverse=True)

    def delete(self, item_id: str) -> bool:
        return self._items.pop(item_id, None) is not None

    def count(self):
        return len(self._items)

    def stats(self):
        total = len(self._items)
        done = sum(1 for i in self._items.values() if i.done)
        overdue = sum(1 for i in self._items.values() if i.is_overdue())
        return {"total": total, "done": done, "pending": total - done, "overdue": overdue}
PYEOF

echo "Demo repo ready at $DEMO_DIR"
echo "Lines in diff: $(cd "$DEMO_DIR" && git diff | wc -l | tr -d ' ')"
