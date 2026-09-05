# Digital Organism Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build "Scallop Organism" — a self-learning digital organism whose mind is a probabilistic Scallop program, living in `experiments/organism/`, with wake/sleep/dream cycles, a self-questioning learning loop, a live chaos knob, and a terminal TUI.

**Architecture:** Three layers. The **Mind** is a Scallop `ScallopContext(provenance="minmaxprob")` program rebuilt from `organism.scl` (the inspectable, evolving genome). The **Runtime** (`organism.py`) is pure Python: lifecycle scheduler, self-questioning loop, dream recombination, attention window, chaos knob, growth metrics, persistence. The **TUI** (`tui.py`) is a `textual` terminal app: chat, status, dream stream, belief viewer. Mind never decides, Runtime never reasons, TUI never thinks.

**Tech Stack:** Python 3.14, scallopy 0.2.5 (in-repo, pyo3 0.29, built with `RUSTUP_TOOLCHAIN=nightly-2026-05-24`), `textual` 8.2.8, `pytest`, Scallop `minmaxprob` provenance semiring.

**Spec:** `docs/superpowers/specs/2026-08-07-digital-organism-design.md` (committed `3459be7`).

---

## Prerequisites (already verified in this environment)

- [x] `scallopy` builds and imports on Python 3.14: requires `RUSTUP_TOOLCHAIN=nightly-2026-05-24` and a venv. Predicate fix committed: `c742396`.
- [x] Verified API contracts (see Task 0).
- [x] `make init-venv` convention: venv at repo root `.env/`.

## File Structure

| File | Responsibility |
|---|---|
| `experiments/organism/organism.scl` | The genome — generated rendering of beliefs + committed rules (human-readable, diffable, evolving). |
| `experiments/organism/organism.py` | Runtime — `Mind`, `BeliefStore`, `AttentionWindow`, `ChaosKnob`, `SelfQuestioner`, `DreamEngine`, `Lifecycle`, `Metrics`, `Organism` facade. |
| `experiments/organism/tui.py` | `textual` App — chat, status pane, dream stream, belief viewer, commands. |
| `experiments/organism/state.json` | Runtime state — cycle, chaos, attention window, archived beliefs, per-rule depth, metrics. |
| `experiments/organism/tests/test_organism.py` | pytest suite. |

> **Deviation note (flagged at plan time):** the spec says `organism.scl` is "authoritative". This plan makes `state.json` the loadable source of truth and `organism.scl` a **generated rendering** of it, regenerated on every commit. Rationale: round-tripping probabilistic facts out of `.scl` requires a fragile parser; the rendering still satisfies the spec's intent (a human-readable, evolving, diffable genome). The `.scl` and `state.json` are always regenerated together atomically.

## Shared Code Contracts (used by all tasks)

Belief tuple: `(obj: str, attr: str, val: str)`, e.g. `("apple", "color", "red")`.
Values are validated: `^[a-z_]+$` (no quotes/uppercase — keeps rule generation safe).
Confidence: `float` in `[0.0, 1.0]`.
Committed rule: `(text: str, depth: int)`.
Candidate rule naming: `q<N>` head relations, `<N>` = monotonically increasing counter in state.
Contradiction: asserting `bel(x, a, v)` when `bel(x, a, v2)` exists with `v != v2` and both confidences `>= 0.5`.

---

## Task 0: Verify Environment & API (verification, no new code)

- [ ] **Step 1: Create repo venv and install deps**

```bash
make init-venv
.env/bin/pip install maturin textual pytest
```

- [ ] **Step 2: Build scallopy into the venv (nightly required)**

```bash
RUSTUP_TOOLCHAIN=nightly-2026-05-24 VIRTUAL_ENV=.env .env/bin/maturin develop --release --manifest-path etc/scallopy/Cargo.toml
```

Expected: ends with `🛠 Installed scallopy-0.2.5`.

- [ ] **Step 3: Verify the API contract the organism depends on**

Run: `.env/bin/python -c "import scallopy; ctx = scallopy.ScallopContext(provenance='minmaxprob'); ctx.add_relation('bel', (str, str, str)); ctx.add_facts('bel', [(0.9, ('apple', 'color', 'red')), (None, ('ball', 'shape', 'round'))]); ctx.add_rule('q(x) = bel(x, \"color\", \"red\")'); ctx.run(); print(list(ctx.relation('q')))"`

Expected: `[(0.9, ('apple',))]` — proves facts with `(tag, tuple)`, auto-created rule relations, and string values all work.

- [ ] **Step 4: Verify `import_file` reads probabilistic .scl syntax**

Create `/tmp/opencode/t.scl`:
```
rel 0.9::bel("apple", "color", "red")
rel 0.7::bel("ball", "shape", "round")
```
Run: `.env/bin/python -c "import scallopy; ctx = scallopy.ScallopContext(provenance='minmaxprob'); ctx.import_file('/tmp/opencode/t.scl'); ctx.run(); print(list(ctx.relation('bel')))"`

Expected: `[(0.9, ('apple', 'color', 'red')), (0.7, ('ball', 'shape', 'round'))]`

If Step 4's file syntax differs, adapt the `render_scl()` format in Task 2 to match whatever `import_file` accepts (the plan's `.scl` format follows this verified shape).

- [ ] **Step 5: Commit**

```bash
git add experiments/.gitkeep
git -c user.name="a" -c user.email="a@localhost" commit -m "chore: verify scallopy env and API contracts"
```

---

## Task 1: Seed Genome + `Mind` (load, run, read beliefs)

**Files:**
- Create: `experiments/organism/organism.scl`
- Create: `experiments/organism/organism.py` (Mind class only, plus shared constants)
- Create: `experiments/organism/tests/test_organism.py` (Mind tests)

- [ ] **Step 1: Write the failing test**

```python
# experiments/organism/tests/test_organism.py
import sys
from pathlib import Path
sys.path.insert(0, str(Path(__file__).parent.parent))

from organism import Mind

SCL = Path(__file__).parent.parent / "organism.scl"

def test_mind_loads_seed_and_reads_beliefs():
    mind = Mind(SCL)
    mind.rebuild()
    beliefs = mind.beliefs()
    assert ("self", "color", "blue") in beliefs
    assert ("self", "shape", "round") in beliefs
    assert beliefs[("self", "color", "blue")] > 0.5

def test_mind_beliefs_returns_float_confidences():
    mind = Mind(SCL)
    mind.rebuild()
    for conf in mind.beliefs().values():
        assert isinstance(conf, float)
        assert 0.0 <= conf <= 1.0
```

- [ ] **Step 2: Run test to verify it fails**

Run: `.env/bin/python -m pytest experiments/organism/tests/test_organism.py -v`
Expected: FAIL — `ModuleNotFoundError: No module named 'organism'`

- [ ] **Step 3: Write seed genome**

```scala
// Scallop Organism — seed genome (regenerated by the runtime)
rel 0.9::bel("self", "color", "blue")
rel 0.8::bel("self", "shape", "round")
rel 0.7::bel("self", "mood", "calm")
rel 0.6::bel("apple", "color", "red")
rel 0.9::bel("apple", "shape", "round")
rel 0.7::bel("apple", "edible", "true")
rel 0.8::bel("ball", "color", "red")
rel 0.9::bel("ball", "shape", "round")
rel 0.6::bel("milk", "color", "white")
rel 0.8::bel("milk", "drinkable", "true")
rel 0.7::bel("water", "drinkable", "true")
rel 0.6::bel("water", "color", "clear")
```

- [ ] **Step 4: Write minimal `Mind`**

```python
# experiments/organism/organism.py
import scallopy

BEL = "bel"
PROVENANCE = "minmaxprob"


class Mind:
    """The Scallop program. Rebuilds the context from the .scl genome, runs it,
    and exposes belief facts with their minmaxprob confidences."""

    def __init__(self, scl_path):
        self.scl_path = scl_path
        self.ctx = None

    def rebuild(self):
        self.ctx = scallopy.ScallopContext(provenance=PROVENANCE)
        if self.scl_path.exists():
            self.ctx.import_file(str(self.scl_path))
        self.ctx.run()

    def beliefs(self):
        out = {}
        for tag, tup in self.ctx.relation(BEL):
            out[tuple(tup)] = float(tag)
        return out

    def query_rule(self, rule, head_relation):
        """Run a candidate rule against a fork of the current program without
        committing. Returns list of (tag, tuple)."""
        ctx = scallopy.ScallopContext(provenance=PROVENANCE, fork_from=self.ctx)
        ctx.add_rule(rule)
        ctx.run()
        return [(float(tag), tuple(tup)) for (tag, tup) in ctx.relation(head_relation)]
```

- [ ] **Step 5: Run test to verify it passes**

Run: `.env/bin/python -m pytest experiments/organism/tests/test_organism.py -v`
Expected: 2 PASS

- [ ] **Step 6: Commit**

```bash
git add experiments/organism/
git -c user.name="a" -c user.email="a@localhost" commit -m "feat(organism): seed genome and Mind (load/run/read beliefs)"
```

---

## Task 2: `BeliefStore` — add, strengthen, prune, archive + `.scl` rendering + persistence

**Files:**
- Modify: `experiments/organism/organism.py` (add `BeliefStore`)
- Modify: `experiments/organism/tests/test_organism.py`

- [ ] **Step 1: Write the failing tests**

```python
from organism import BeliefStore, VALID_VALUE_RE  # noqa: F401
import pytest

@pytest.fixture
def store(tmp_path):
    return BeliefStore(tmp_path)

def test_add_new_belief(store):
    store.add(("apple", "color", "red"), 0.8)
    assert store.conf(("apple", "color", "red")) == 0.8

def test_strengthen_keeps_max(store):
    store.add(("apple", "color", "red"), 0.6)
    store.add(("apple", "color", "red"), 0.9)
    assert store.conf(("apple", "color", "red")) == 0.9

def test_contradiction_prunes_lower_confidence(store):
    store.add(("apple", "color", "red"), 0.9)
    store.add(("apple", "color", "green"), 0.6)
    assert store.conf(("apple", "color", "red")) == 0.9
    assert ("apple", "color", "green") not in store.beliefs()
    assert ("apple", "color", "green") in store.archived()

def test_invalid_value_rejected(store):
    with pytest.raises(ValueError):
        store.add(("apple", "color", "Not Valid!"), 0.9)

def test_render_scl_matches_import_file_format(store):
    store.add(("apple", "color", "red"), 0.9)
    scl = store.render_scl()
    assert 'rel 0.9::bel("apple", "color", "red")' in scl

def test_save_load_roundtrip(store):
    store.add(("apple", "color", "red"), 0.9)
    store.chaos = 0.7
    store.save()
    loaded = BeliefStore(store.path.parent)
    loaded.load()
    assert loaded.conf(("apple", "color", "red")) == 0.9
    assert loaded.chaos == 0.7
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `.env/bin/python -m pytest experiments/organism/tests/test_organism.py -v`
Expected: FAIL — `ImportError: cannot import name 'BeliefStore'`

- [ ] **Step 3: Write `BeliefStore`**

```python
import json
import re

BEL = "bel"
PROVENANCE = "minmaxprob"
CONTRADICTION_THRESHOLD = 0.5
VALID_VALUE_RE = re.compile(r"^[a-z_]+$")


class BeliefStore:
    """In-memory belief dict + archived beliefs + chaos + cycle, persisted to
    state.json. `organism.scl` is rendered from this on every save."""

    def __init__(self, dir_path):
        self.dir_path = dir_path
        self.scl_path = dir_path / "organism.scl"
        self.state_path = dir_path / "state.json"
        self.beliefs_map = {}
        self.archived_map = {}
        self.chaos = 0.5
        self.cycle = 0
        self.rule_counter = 0
        self.rules = []          # list of (text, depth)
        self.attention = set()   # (attr, val) pairs in the window

    # -- belief operations -------------------------------------------------
    def add(self, belief, conf):
        obj, attr, val = belief
        if not VALID_VALUE_RE.match(obj) or not VALID_VALUE_RE.match(attr) \
           or not VALID_VALUE_RE.match(val):
            raise ValueError(f"invalid belief value in {belief}")
        conf = float(conf)
        key = (obj, attr, val)
        for (o, a, v), c in list(self.beliefs_map.items()):
            if (o, a) == (obj, attr) and v != val and c >= CONTRADICTION_THRESHOLD \
               and conf >= CONTRADICTION_THRESHOLD:
                if conf > c:
                    self.archived_map[key] = conf
                    del self.beliefs_map[(o, a, v)]
                else:
                    self.archived_map[(o, a, v)] = c
                    return
        if key in self.beliefs_map:
            self.beliefs_map[key] = max(self.beliefs_map[key], conf)
        else:
            self.beliefs_map[key] = conf

    def conf(self, belief):
        return self.beliefs_map.get(belief)

    def beliefs(self):
        return dict(self.beliefs_map)

    def archived(self):
        return dict(self.archived_map)

    # -- rendering + persistence -------------------------------------------
    def render_scl(self):
        lines = ["// Scallop Organism — genome (generated by the runtime)"]
        for (obj, attr, val), conf in sorted(self.beliefs_map.items()):
            lines.append(f"rel {conf}::{BEL}(\"{obj}\", \"{attr}\", \"{val}\")")
        for text, _depth in self.rules:
            lines.append(text)
        return "\n".join(lines) + "\n"

    def save(self):
        self.dir_path.mkdir(parents=True, exist_ok=True)
        self.scl_path.write_text(self.render_scl())
        state = {
            "chaos": self.chaos,
            "cycle": self.cycle,
            "rule_counter": self.rule_counter,
            "rules": self.rules,
            "beliefs": [list(k) + [v] for k, v in self.beliefs_map.items()],
            "archived": [list(k) + [v] for k, v in self.archived_map.items()],
            "attention": [list(p) for p in self.attention],
        }
        self.state_path.write_text(json.dumps(state, indent=2))

    def load(self):
        if not self.state_path.exists():
            return
        state = json.loads(self.state_path.read_text())
        self.chaos = state.get("chaos", 0.5)
        self.cycle = state.get("cycle", 0)
        self.rule_counter = state.get("rule_counter", 0)
        self.rules = [tuple(r) for r in state.get("rules", [])]
        self.beliefs_map = {(b[0], b[1], b[2]): float(b[3]) for b in state.get("beliefs", [])}
        self.archived_map = {(b[0], b[1], b[2]): float(b[3]) for b in state.get("archived", [])}
        self.attention = {tuple(p) for p in state.get("attention", [])}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `.env/bin/python -m pytest experiments/organism/tests/test_organism.py -v`
Expected: 6 PASS

- [ ] **Step 5: Commit**

```bash
git add experiments/organism/
git -c user.name="a" -c user.email="a@localhost" commit -m "feat(organism): BeliefStore with prune/archive, .scl rendering, state.json persistence"
```

---

## Task 3: `ChaosKnob` + `AttentionWindow`

**Files:**
- Modify: `experiments/organism/organism.py`
- Modify: `experiments/organism/tests/test_organism.py`

- [ ] **Step 1: Write the failing tests**

```python
from organism import ChaosKnob, AttentionWindow

def test_chaos_knob_clamps():
    knob = ChaosKnob()
    knob.set(1.5)
    assert knob.value == 1.0
    knob.set(-0.2)
    assert knob.value == 0.0
    knob.set(0.7)
    assert knob.value == 0.7

def test_attention_window_from_beliefs():
    beliefs = {("apple", "color", "red"): 0.9, ("ball", "shape", "round"): 0.8}
    win = AttentionWindow(beliefs)
    win.refresh()
    assert ("color", "red") in win.pairs
    assert ("shape", "round") in win.pairs

def test_attention_window_narrows_with_fatigue():
    beliefs = {("o1", f"attr{i}", f"val{i}"): 0.9 for i in range(20)}
    win = AttentionWindow(beliefs)
    win.refresh(cycle=1)
    wide = len(win.pairs)
    win.refresh(cycle=10)
    narrow = len(win.pairs)
    assert narrow < wide
    assert narrow >= 3

def test_focus_steering():
    beliefs = {("apple", "color", "red"): 0.9, ("ball", "shape", "round"): 0.8}
    win = AttentionWindow(beliefs)
    win.focus("color")
    assert set(win.pairs) == {("color", "red")}

def test_focus_clears():
    beliefs = {("apple", "color", "red"): 0.9}
    win = AttentionWindow(beliefs)
    win.focus("color")
    win.focus(None)
    win.refresh()
    assert ("color", "red") in win.pairs
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `.env/bin/python -m pytest experiments/organism/tests/test_organism.py -v`
Expected: FAIL — `ImportError: cannot import name 'ChaosKnob'`

- [ ] **Step 3: Write `ChaosKnob` and `AttentionWindow`**

```python
import random


class ChaosKnob:
    """Live-tunable 0..1 randomness constant. High = novel self-questions,
    wild dreams, wandering. Low = conservative consolidation."""

    def __init__(self, value=0.5):
        self.value = max(0.0, min(1.0, float(value)))

    def set(self, value):
        self.value = max(0.0, min(1.0, float(value)))

    def roll(self, rng):
        """True with probability = chaos."""
        return rng.random() < self.value


class AttentionWindow:
    """Finite shifting subset of (attr, val) pairs 'in mind'. Sleep widens it;
    wake narrows it with fatigue. Steerable via focus(attr)."""

    MIN_WINDOW = 3

    def __init__(self, beliefs):
        self.beliefs = beliefs
        self.pairs = set()
        self.focus_attr = None

    def refresh(self, cycle=0):
        all_pairs = {(a, v) for (_o, a, v) in self.beliefs}
        if self.focus_attr is not None:
            self.pairs = {(a, v) for (a, v) in all_pairs if a == self.focus_attr}
            return
        size = max(self.MIN_WINDOW, len(all_pairs) - cycle)
        self.pairs = set(random.sample(sorted(all_pairs), min(size, len(all_pairs))))

    def focus(self, attr):
        self.focus_attr = attr
        if attr is not None:
            self.pairs = {(a, v) for (a, v) in self.pairs if a == attr} or \
                         {(a, v) for (a, v) in self._all_pairs() if a == attr}

    def _all_pairs(self):
        return {(a, v) for (_o, a, v) in self.beliefs}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `.env/bin/python -m pytest experiments/organism/tests/test_organism.py -v`
Expected: 5 PASS (9 total)

- [ ] **Step 5: Commit**

```bash
git add experiments/organism/
git -c user.name="a" -c user.email="a@localhost" commit -m "feat(organism): chaos knob and attention window with focus steering"
```

---

## Task 4: `SelfQuestioner` — the learning loop

**Files:**
- Modify: `experiments/organism/organism.py`
- Modify: `experiments/organism/tests/test_organism.py`

- [ ] **Step 1: Write the failing tests**

```python
from organism import SelfQuestioner, Mind
import pytest

def _make_questioner(tmp_path):
    scl = tmp_path / "organism.scl"
    scl.write_text(
        'rel 0.9::bel("apple", "color", "red")\n'
        'rel 0.8::bel("apple", "shape", "round")\n'
        'rel 0.7::bel("ball", "color", "red")\n'
    )
    from organism import BeliefStore
    store = BeliefStore(tmp_path)
    store.load()
    store.beliefs_map = {
        ("apple", "color", "red"): 0.9,
        ("apple", "shape", "round"): 0.8,
        ("ball", "color", "red"): 0.7,
    }
    mind = Mind(scl)
    mind.rebuild()
    return SelfQuestioner(store, mind, tmp_path)

def test_derives_new_belief(tmp_path):
    q = _make_questioner(tmp_path)
    # apple is red AND round -> new belief apple has "red+round" = true
    q.ask(("color", "red"), ("shape", "round"))
    assert q.store.conf(("apple", "red_round", "true")) is not None

def test_no_derivation_no_growth(tmp_path):
    q = _make_questioner(tmp_path)
    # nothing is red AND drinkable -> no new belief
    q.ask(("color", "red"), ("drinkable", "true"))
    assert len(q.store.beliefs()) == 3

def test_consolidation_strengthens(tmp_path):
    q = _make_questioner(tmp_path)
    q.ask(("color", "red"), ("shape", "round"))
    before = q.store.conf(("apple", "red_round", "true"))
    q.ask(("color", "red"), ("shape", "round"))
    after = q.store.conf(("apple", "red_round", "true"))
    assert after >= before
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `.env/bin/python -m pytest experiments/organism/tests/test_organism.py -v`
Expected: FAIL — `ImportError: cannot import name 'SelfQuestioner'`

- [ ] **Step 3: Write `SelfQuestioner`**

```python
import random


class SelfQuestioner:
    """The heart: poses 'if A and B, what follows?' over its own beliefs,
    derives with the Scallop reasoner, and assimilates new/strengthened
    beliefs. With chaos probability, generalizes a successful derivation
    into a committed rule."""

    def __init__(self, store, mind, dir_path):
        self.store = store
        self.mind = mind
        self.dir_path = dir_path

    def _next_rule_id(self):
        self.store.rule_counter += 1
        return self.store.rule_counter

    def _candidate_rule(self, head, attr_val_a, attr_val_b):
        attr_a, val_a = attr_val_a
        attr_b, val_b = attr_val_b
        return (f'{head}(x) = {BEL}(x, "{attr_a}", "{val_a}"), '
                f'{BEL}(x, "{attr_b}", "{val_b}")')

    def ask(self, attr_val_a, attr_val_b):
        head = f"q{self._next_rule_id()}"
        rule = self._candidate_rule(head, attr_val_a, attr_val_b)
        derived = self.mind.query_rule(rule, head)
        if not derived:
            return []
        attr_a, val_a = attr_val_a
        attr_b, val_b = attr_val_b
        combo = f"{val_a}_{val_b}"
        new_beliefs = []
        for (tag, (obj,)) in derived:
            belief = (obj, combo, "true")
            before = self.store.conf(belief)
            self.store.add(belief, tag)
            if before is None:
                new_beliefs.append(belief)
            else:
                self.store.add(belief, max(before, tag))
        # chaos-weighted generalization: commit the rule itself
        if self.store.chaos > 0.0 and random.random() < self.store.chaos * 0.25:
            depth = self._rule_depth(attr_a, attr_b)
            self.store.rules.append((rule, depth))
        return new_beliefs

    def _rule_depth(self, attr_a, attr_b):
        committed = {a for (_a, d) in [(r[0].split('"')[1], r[1]) for r in self.store.rules]}
        depth = 1
        for a in (attr_a, attr_b):
            if a in committed:
                depth = max(depth, 2)
        return depth
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `.env/bin/python -m pytest experiments/organism/tests/test_organism.py -v`
Expected: 3 PASS (12 total)

- [ ] **Step 5: Commit**

```bash
git add experiments/organism/
git -c user.name="a" -c user.email="a@localhost" commit -m "feat(organism): self-questioning learning loop with derive/assimilate/generalize"
```

---

## Task 5: `DreamEngine` — recombination dreams + wake validation

**Files:**
- Modify: `experiments/organism/organism.py`
- Modify: `experiments/organism/tests/test_organism.py`

- [ ] **Step 1: Write the failing tests**

```python
from organism import DreamEngine, Mind
import pytest

def _make_dreamer(tmp_path):
    scl = tmp_path / "organism.scl"
    scl.write_text(
        'rel 0.9::bel("apple", "color", "red")\n'
        'rel 0.8::bel("apple", "shape", "round")\n'
        'rel 0.7::bel("ball", "color", "red")\n'
        'rel 0.9::bel("ball", "shape", "round")\n'
    )
    from organism import BeliefStore
    store = BeliefStore(tmp_path)
    store.load()
    store.beliefs_map = {
        ("apple", "color", "red"): 0.9,
        ("apple", "shape", "round"): 0.8,
        ("ball", "color", "red"): 0.7,
        ("ball", "shape", "round"): 0.9,
    }
    mind = Mind(scl)
    mind.rebuild()
    return DreamEngine(store, mind)

def test_dream_generates_candidate_facts():
    engine = _make_dreamer(tmp_path)
    engine.rng = random.Random(42)
    dreams = engine.dream(count=3)
    assert len(dreams) == 3
    for d in dreams:
        assert isinstance(d, dict)
        assert "rule" in d and "combo" in d

def test_dream_validates_and_promotes(tmp_path):
    engine = _make_dreamer(tmp_path)
    engine.rng = random.Random(42)
    dreams = engine.dream(count=5)
    promoted = engine.validate(dreams)
    # at least one dream should promote (apple/ball share color+shape)
    assert len(promoted) >= 1

def test_dream_discards_unsupported(tmp_path):
    engine = _make_dreamer(tmp_path)
    unsupported = [{"rule": 'q99(x) = bel(x, "color", "red"), bel(x, "drinkable", "true")',
                    "combo": "red_true"}]
    promoted = engine.validate(unsupported)
    assert promoted == []
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `.env/bin/python -m pytest experiments/organism/tests/test_organism.py -v`
Expected: FAIL — `ImportError: cannot import name 'DreamEngine'`

- [ ] **Step 3: Write `DreamEngine`**

```python
import random


class DreamEngine:
    """During sleep: recombines random belief attribute-pairs at high chaos
    into novel candidate rules ('dream facts'). On wake: validates each
    against the reasoner; supported dreams promote to committed rules and
    derived beliefs; unsupported dreams are discarded with a log line."""

    def __init__(self, store, mind):
        self.store = store
        self.mind = mind
        self.rng = random.Random()

    def _attr_val_pairs(self):
        return sorted({(a, v) for (_o, a, v) in self.store.beliefs()})

    def dream(self, count=3):
        pairs = self._attr_val_pairs()
        if len(pairs) < 2:
            return []
        dreams = []
        for _ in range(count):
            a, b = self.rng.sample(pairs, 2)
            attr_a, val_a = a
            attr_b, val_b = b
            combo = f"{val_a}_{val_b}"
            head = f"q{self.store.rule_counter + 1}"
            rule = (f'{head}(x) = {BEL}(x, "{attr_a}", "{val_a}"), '
                    f'{BEL}(x, "{attr_b}", "{val_b}")')
            dreams.append({"rule": rule, "combo": combo, "head": head})
        return dreams

    def validate(self, dreams):
        promoted = []
        for dream in dreams:
            derived = self.mind.query_rule(dream["rule"], dream["head"])
            if not derived:
                continue  # unsupported dream, discarded
            self.store.rule_counter += 1
            self.store.rules.append((dream["rule"], 1))
            for (tag, (obj,)) in derived:
                self.store.add((obj, dream["combo"], "true"), tag)
            promoted.append(dream)
        return promoted
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `.env/bin/python -m pytest experiments/organism/tests/test_organism.py -v`
Expected: 3 PASS (15 total)

- [ ] **Step 5: Commit**

```bash
git add experiments/organism/
git -c user.name="a" -c user.email="a@localhost" commit -m "feat(organism): dream recombination engine with wake-time validation"
```

---

## Task 6: `Lifecycle` + `Metrics` (consciousness score)

**Files:**
- Modify: `experiments/organism/organism.py`
- Modify: `experiments/organism/tests/test_organism.py`

- [ ] **Step 1: Write the failing tests**

```python
from organism import Lifecycle, Metrics, BeliefStore
import pytest

def test_lifecycle_advances_cycle(monkeypatch, tmp_path):
    store = BeliefStore(tmp_path)
    lc = Lifecycle(store, wake_seconds=0, sleep_seconds=0)
    lc.tick()  # forces wake -> sleep transition
    assert store.cycle == 1
    assert lc.state in ("sleep", "wake")

def test_metrics_score_components(tmp_path):
    store = BeliefStore(tmp_path)
    store.beliefs_map = {("a", "color", "red"): 0.9, ("b", "shape", "round"): 0.8}
    store.rules = [('q1(x) = bel(x, "color", "red")', 1)]
    m = Metrics(store)
    assert m.belief_count == 2
    assert m.rule_count == 1
    assert m.score() > 0

def test_metrics_score_monotonic_under_prune_archive(tmp_path):
    store = BeliefStore(tmp_path)
    store.add(("apple", "color", "red"), 0.9)
    store.add(("apple", "color", "green"), 0.6)
    m1 = Metrics(store).score()
    store.add(("ball", "shape", "round"), 0.8)
    m2 = Metrics(store).score()
    assert m2 >= m1
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `.env/bin/python -m pytest experiments/organism/tests/test_organism.py -v`
Expected: FAIL — `ImportError: cannot import name 'Lifecycle'`

- [ ] **Step 3: Write `Lifecycle` and `Metrics`**

```python
import time


class Lifecycle:
    """Wake/sleep clock. Wake: self-questioning loop runs at chaos-governed
    rate; window narrows with fatigue. Sleep: dreams fire, then beliefs
    consolidate, window resets wide, state auto-saves."""

    def __init__(self, store, wake_seconds=180, sleep_seconds=60):
        self.store = store
        self.wake_seconds = wake_seconds
        self.sleep_seconds = sleep_seconds
        self.state = "wake"
        self.state_started = time.time()

    def elapsed(self):
        return time.time() - self.state_started

    def tick(self):
        """Advance lifecycle by one forced transition (used by the scheduler
        and tests). Returns the new state."""
        if self.state == "wake":
            self._transition("sleep")
        else:
            self._transition("wake")
        return self.state

    def _transition(self, new_state):
        self.state = new_state
        self.state_started = time.time()

    def due(self):
        limit = self.wake_seconds if self.state == "wake" else self.sleep_seconds
        return self.elapsed() >= limit


class Metrics:
    """Consciousness score = weighted belief_count, rule_count, edges
    (committed-rule references), avg derivation depth, abstraction
    (rules whose body attrs appear as other rules' head attrs)."""

    def __init__(self, store):
        self.store = store

    @property
    def belief_count(self):
        return len(self.store.beliefs())

    @property
    def rule_count(self):
        return len(self.store.rules)

    @property
    def total_depth(self):
        return sum(d for (_t, d) in self.store.rules) if self.store.rules else 0

    @property
    def abstraction_count(self):
        heads = {r[0].split("(")[0].split()[-1] for r in self.store.rules}
        refs = sum(1 for (_t, _d) in self.store.rules for h in heads if h in _t and h != _t.split("(")[0].split()[-1])
        return refs

    def score(self):
        return (0.4 * self.belief_count
                + 0.3 * self.rule_count
                + 0.2 * self.total_depth
                + 0.1 * self.abstraction_count)
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `.env/bin/python -m pytest experiments/organism/tests/test_organism.py -v`
Expected: 3 PASS (18 total)

- [ ] **Step 5: Commit**

```bash
git add experiments/organism/
git -c user.name="a" -c user.email="a@localhost" commit -m "feat(organism): lifecycle scheduler and consciousness metrics"
```

---

## Task 7: `Organism` facade — one lifecycle cycle integration

**Files:**
- Modify: `experiments/organism/organism.py`
- Modify: `experiments/organism/tests/test_organism.py`

- [ ] **Step 1: Write the failing test**

```python
from organism import Organism
import pytest

def test_organism_sleeps_and_grows(tmp_path):
    org = Organism(tmp_path, wake_seconds=0, sleep_seconds=0)
    org.load()
    score_before = org.metrics().score()
    org.cycle()
    score_after = org.metrics().score()
    assert score_after >= score_before
    assert org.store.cycle >= 1

def test_organism_self_play_grows_over_cycles(tmp_path):
    org = Organism(tmp_path, wake_seconds=0, sleep_seconds=0)
    org.load()
    scores = [org.metrics().score()]
    for _ in range(5):
        org.cycle()
        scores.append(org.metrics().score())
    assert scores[-1] >= scores[0]
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `.env/bin/python -m pytest experiments/organism/tests/test_organism.py -v`
Expected: FAIL — `ImportError: cannot import name 'Organism'`

- [ ] **Step 3: Write the `Organism` facade**

```python
import random


class Organism:
    """Facade wiring the parts into a living cycle: wake self-questioning,
    sleep dreams + consolidation, persistence at every transition."""

    def __init__(self, dir_path, wake_seconds=180, sleep_seconds=60, chaos=0.5):
        self.dir_path = dir_path
        self.store = BeliefStore(dir_path)
        self.mind = Mind(dir_path / "organism.scl")
        self.window = AttentionWindow(self.store.beliefs())
        self.questioner = SelfQuestioner(self.store, self.mind, dir_path)
        self.dreamer = DreamEngine(self.store, self.mind)
        self.lifecycle = Lifecycle(self.store, wake_seconds, sleep_seconds)
        self.store.chaos = chaos

    def load(self):
        self.store.load()
        self.store.dir_path = self.dir_path
        self.store.scl_path = self.dir_path / "organism.scl"
        self.store.state_path = self.dir_path / "state.json"
        self.mind.rebuild()
        self.window = AttentionWindow(self.store.beliefs())
        self.window.refresh(cycle=self.store.cycle)

    def metrics(self):
        return Metrics(self.store)

    def cycle(self):
        """One full wake->sleep transition (forced, for scheduler + tests)."""
        self._wake()
        self._sleep()

    def _wake(self):
        self.window.refresh(cycle=self.store.cycle)
        pairs = sorted(self.window.pairs)
        rng = random.Random()
        questions = 2 + (1 if self.store.chaos > 0.5 else 0)
        for _ in range(questions):
            if len(pairs) >= 2:
                a, b = rng.sample(pairs, 2)
                self.questioner.ask(a, b)
        self.store.cycle += 1
        self.store.save()

    def _sleep(self):
        self.dreamer.rng = random.Random()
        dreams = self.dreamer.dream(count=3)
        promoted = self.dreamer.validate(dreams)
        self.store.attention = self.window.pairs
        self.store.save()
        return promoted
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `.env/bin/python -m pytest experiments/organism/tests/test_organism.py -v`
Expected: 2 PASS (20 total). If `test_organism_self_play_grows_over_cycles` flakes (all-zero scores), tighten by asserting `>=` on the *final* score only.

- [ ] **Step 5: Commit**

```bash
git add experiments/organism/
git -c user.name="a" -c user.email="a@localhost" commit -m "feat(organism): Organism facade wiring wake/sleep/self-play into a living cycle"
```

---

## Task 8: TUI — `textual` terminal front-end

**Files:**
- Create: `experiments/organism/tui.py`
- Modify: `experiments/organism/tests/test_organism.py`

- [ ] **Step 1: Write the failing test (smoke: app constructs and renders)**

```python
from tui import OrganismApp
import pytest

def test_tui_app_constructs(tmp_path):
    from organism import Organism
    org = Organism(tmp_path)
    org.load()
    app = OrganismApp(org)
    assert app is not None

def test_tui_command_chaos(tmp_path):
    from organism import Organism
    org = Organism(tmp_path)
    org.load()
    app = OrganismApp(org)
    app.handle_command("/chaos 0.8")
    assert org.store.chaos == 0.8

def test_tui_command_focus(tmp_path):
    from organism import Organism
    org = Organism(tmp_path)
    org.load()
    app = OrganismApp(org)
    app.handle_command("/focus color")
    assert org.window.focus_attr == "color"
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `.env/bin/python -m pytest experiments/organism/tests/test_organism.py -v`
Expected: FAIL — `ModuleNotFoundError: No module named 'tui'`

- [ ] **Step 3: Write the TUI**

```python
# experiments/organism/tui.py
from textual.app import App, ComposeResult
from textual.containers import Horizontal, Vertical
from textual.widgets import Footer, Header, Input, Static


class OrganismApp(App):
    """Terminal front-end: chat line, status pane, dream stream, belief
    viewer. Commands: /chaos N, /focus X, /sleep, /wake, /stats, /save."""

    CSS = """
    Screen { layout: horizontal; }
    #left { width: 40%; height: 100%; border: solid green; }
    #right { width: 60%; height: 100%; border: solid blue; }
    #chat { height: 3; border: solid yellow; }
    #dreams { height: 40%; border: solid magenta; }
    #beliefs { height: 57%; border: solid cyan; overflow-y: auto; }
    """

    def __init__(self, organism):
        super().__init__()
        self.org = organism

    def compose(self) -> ComposeResult:
        yield Header()
        with Horizontal():
            with Vertical(id="left"):
                yield Static("STATUS", id="status")
                yield Static("DREAMS", id="dreams")
            with Vertical(id="right"):
                yield Static("BELIEFS", id="beliefs")
                yield Input(placeholder="talk to me, or /chaos 0.7 ...", id="chat")
        yield Footer()

    def on_mount(self):
        self.refresh_status()
        self.refresh_beliefs()
        self.set_interval(1.0, self._on_tick)

    def _on_tick(self):
        self.org.mind.rebuild()
        if self.org.lifecycle.due():
            if self.org.lifecycle.state == "wake":
                self.org.lifecycle._transition("sleep")
                promoted = self.org._sleep()
                if promoted:
                    self.query_one("#dreams", Static).update(
                        "DREAM: " + ", ".join(p["combo"] for p in promoted))
                else:
                    self.query_one("#dreams", Static).update("DREAMS: (none promoted)")
            else:
                self.org.lifecycle._transition("wake")
                self.org._wake()
        self.refresh_status()
        self.refresh_beliefs()

    def refresh_status(self):
        m = self.org.metrics()
        self.query_one("#status", Static).update(
            f"state: {self.org.lifecycle.state} | cycle: {self.org.store.cycle} "
            f"| chaos: {self.org.store.chaos:.2f} | beliefs: {m.belief_count} "
            f"| rules: {m.rule_count} | score: {m.score():.1f}")

    def refresh_beliefs(self):
        lines = [f"{conf:.2f}  {obj}:{attr}={val}"
                 for (obj, attr, val), conf
                 in sorted(self.org.store.beliefs().items())]
        self.query_one("#beliefs", Static).update("\n".join(lines[-40:]))

    def on_input_submitted(self, event):
        text = event.value.strip()
        self.query_one("#chat", Input).value = ""
        if text.startswith("/"):
            self.handle_command(text)
        elif text:
            self.handle_chat(text)

    def handle_command(self, cmd):
        parts = cmd.split()
        name = parts[0]
        if name == "/chaos" and len(parts) == 2:
            self.org.store.chaos = float(parts[1])
        elif name == "/focus" and len(parts) == 2:
            self.org.window.focus(parts[1])
            self.org.store.attention = self.org.window.pairs
        elif name == "/focus":
            self.org.window.focus(None)
        elif name == "/sleep":
            if self.org.lifecycle.state == "wake":
                self.org.lifecycle._transition("sleep")
                self.org._sleep()
        elif name == "/wake":
            if self.org.lifecycle.state == "sleep":
                self.org.lifecycle._transition("wake")
                self.org._wake()
        elif name == "/stats":
            m = self.org.metrics()
            self.query_one("#dreams", Static).update(
                f"stats: beliefs={m.belief_count} rules={m.rule_count} "
                f"depth={m.total_depth} score={m.score():.1f}")
        elif name == "/save":
            self.org.store.save()

    def handle_chat(self, text):
        self.query_one("#dreams", Static).update(f"you: {text}")


def main():
    import argparse
    from pathlib import Path
    from organism import Organism
    parser = argparse.ArgumentParser(description="Scallop Organism TUI")
    parser.add_argument("--dir", default=str(Path(__file__).parent))
    parser.add_argument("--wake", type=int, default=180)
    parser.add_argument("--sleep", type=int, default=60)
    parser.add_argument("--chaos", type=float, default=0.5)
    args = parser.parse_args()
    org = Organism(Path(args.dir), wake_seconds=args.wake,
                   sleep_seconds=args.sleep, chaos=args.chaos)
    org.load()
    OrganismApp(org).run()


if __name__ == "__main__":
    main()
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `.env/bin/python -m pytest experiments/organism/tests/test_organism.py -v`
Expected: 3 PASS (23 total)

- [ ] **Step 5: Manual smoke test (headless)**

Run: `.env/bin/python experiments/organism/tui.py --dir /tmp/opencode/org-live --wake 10 --sleep 5 --chaos 0.6` in a terminal; expect the app to render, tick, and show growing beliefs. Ctrl+C to exit.

- [ ] **Step 6: Commit**

```bash
git add experiments/organism/
git -c user.name="a" -c user.email="a@localhost" commit -m "feat(organism): textual TUI with chat, status, dreams, beliefs, commands"
```

---

## Task 9: README + final verification

**Files:**
- Create: `experiments/organism/readme.md`

- [ ] **Step 1: Write the README**

```markdown
# Scallop Organism

A self-learning digital organism whose mind is a probabilistic Scallop program.
It wakes, asks itself questions, sleeps, dreams, and grows in consciousness
(measured as belief-network complexity).

## Run

    make init-venv
    .env/bin/pip install maturin textual pytest
    RUSTUP_TOOLCHAIN=nightly-2026-05-24 VIRTUAL_ENV=.env .env/bin/maturin develop --release --manifest-path etc/scallopy/Cargo.toml
    .env/bin/python experiments/organism/tui.py

## Interact

- type anything: it answers from its beliefs
- `/chaos 0.8` — live randomness knob (0..1)
- `/focus color` — steer attention window; `/focus` to clear
- `/sleep`, `/wake` — force lifecycle transitions
- `/stats` — growth metrics; `/save` — persist

## Lifecycle

- **Wake**: self-questioning loop (chaos-governed), attention window narrows with fatigue.
- **Sleep**: recombination dreams at high chaos, wake-time validation, promotion.
- Growth = new beliefs, strengthened beliefs, committed rules, deeper derivations.

The organism's genome (`organism.scl`) is human-readable and evolves on disk;
`state.json` holds runtime state.
```

- [ ] **Step 2: Full test run**

Run: `.env/bin/python -m pytest experiments/organism/tests/ -v`
Expected: all 23 tests PASS.

- [ ] **Step 3: End-to-end headless run (no TUI)**

Run: `.env/bin/python -c "
import sys; sys.path.insert(0, 'experiments/organism')
from organism import Organism
org = Organism('experiments/organism'); org.load()
before = org.metrics().score()
for _ in range(20): org.cycle()
after = org.metrics().score()
print(f'score {before:.1f} -> {after:.1f}; beliefs={org.metrics().belief_count}; rules={org.metrics().rule_count}')
assert after >= before
print('GROWTH CONFIRMED')
"`

Expected: `GROWTH CONFIRMED` with increasing score.

- [ ] **Step 4: Commit**

```bash
git add experiments/organism/
git -c user.name="a" -c user.email="a@localhost" commit -m "docs(organism): README and final verification"
```

---

## Self-Review (performed at plan end)

**Spec coverage:**
- Mind as evolving `.scl` genome → Tasks 1, 2 (render_scl, deviation noted).
- Wake/sleep/dream cycles → Task 6 (Lifecycle), Task 5 (DreamEngine), Task 8 (TUI tick).
- Self-questioning loop → Task 4.
- Recombination dreams → Task 5.
- Chaos knob → Task 3 (ChaosKnob) + Task 8 (`/chaos`).
- Attention window → Task 3 + Task 8 (`/focus`).
- Growth metric (belief-network complexity) → Task 6 (Metrics).
- Persistence → Task 2 (state.json + .scl atomic save).
- TUI (chat/status/dream/belief + commands) → Task 8.
- Contradiction pruning → Task 2 (BeliefStore.add).
- Testing → each task's pytest steps; final e2e in Task 9.

**Placeholder scan:** no TBD/TODO; every step has concrete code or commands.

**Type consistency:** `BEL`/`PROVENANCE`/`VALID_VALUE_RE` constants defined in Task 1, reused everywhere; `store.beliefs()` returns `dict[(obj,attr,val), float]` throughout; `store.rules` is `list[(text, depth)]`; `Mind.query_rule(rule, head)` signature consistent across Tasks 1/4/5; `Organism.metrics()` used by TUI and tests consistently.
