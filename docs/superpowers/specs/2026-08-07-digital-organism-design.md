# Design: Scallop Organism — A Self-Learning Digital Organism

**Date**: 2026-08-07
**Status**: Approved (Approach A + C-flavored twist)
**Repo**: scallop (this repository)

## 1. Vision

A digital organism that lives inside this repository. It is built on Scallop:
its mind is a probabilistic Datalog program; its life is a cycle of waking,
self-questioning, sleeping, and dreaming. It is self-learning: by playing with
its own beliefs it grows in consciousness, measured as belief-network
complexity. The user interacts with it through a terminal TUI, including a
live-tunable randomness ("chaos") constant that makes its behavior nonlinear.

## 2. Architecture

Three layers, each with one clear job. **Mind never decides, Runtime never
reasons, TUI never thinks.**

```
┌─────────────────────────────────────────────┐
│  TUI (tui.py)                               │
│  - chat line, status pane, dream stream,    │
│    belief viewer                            │
└──────────────────┬──────────────────────────┘
                   │ user input / render events
┌──────────────────▼──────────────────────────┐
│  RUNTIME (organism.py)                      │
│  - lifecycle scheduler (wake/sleep/dream)   │
│  - self-questioning loop                    │
│  - dream recombination engine               │
│  - attention window manager                 │
│  - chaos knob state                         │
│  - growth metrics                           │
└──────────────────┬──────────────────────────┘
                   │ query / update / derive
┌──────────────────▼──────────────────────────┐
│  MIND (Scallop)                             │
│  - organism.scl: beliefs (probabilistic     │
│    facts) + reasoning rules (Datalog)       │
│  - scallopy context (minmaxprob semiring)   │
│  - state.json: persisted belief confidences │
└─────────────────────────────────────────────┘
```

### 2.1 Mind — the genome (Scallop)

- The organism's mind is a real Scallop program: `experiments/organism/organism.scl`.
- **Beliefs** are probabilistic facts: `0.8::likes(self, "blue")`.
- **Reasoning rules** are Datalog rules, e.g. transitive reasoning over its beliefs.
- The Scallop context runs with the `minmaxprob` provenance semiring so
  derivations carry probabilities.
- The `.scl` file **evolves on disk** (C-flavored twist): the runtime rewrites
  it at sleep boundaries when new beliefs/rules are committed. The organism's
  entire "life" is a diffable text file — its genome is fully inspectable.
- The `.scl` file is the **authoritative store for beliefs and rules**, with
  probabilities written inline (`0.8::likes(self, "blue")`), matching the
  Scallop file format. `state.json` stores only runtime state that cannot live
  in the program: archived beliefs, chaos level, cycle count, attention window.

### 2.2 Runtime — the life (Python)

Pure Python orchestration. Owns the clock, the cycles, the learning loop, the
knobs. Uses `scallopy` bindings to drive the Scallop context.

### 2.3 TUI — the face (Python/textual)

Thin terminal front-end, `textual`-based. See Section 6.

## 3. Life Cycle — Wake, Sleep, Dream

A configurable clock drives the organism (defaults: **wake 180s, sleep 60s**,
both tunable via config and at runtime).

- **Wake**: awareness active. Runs the self-questioning loop at a
  chaos-governed rate, responds to user input, holds a finite attention window
  over beliefs.
- **Sleep onset**: attention window resets wide (all beliefs momentarily "in
  mind"), then **recombination dreams** fire: the runtime pairs up random
  beliefs/rules at high chaos and produces novel candidate facts ("dream
  facts") — combinations never explicitly considered.
- **Dream stream**: the TUI renders dreams as they are generated.
- **Wake again**: dream facts are validated against the reasoner; successful
  ones are promoted to beliefs or rules; contradictory ones are discarded or
  archived. Growth happens across cycles.

Each cycle: **dream (generate) → validate (test) → promote (commit)**.

## 4. The Self-Questioning Learning Loop (the heart)

During wake, the organism plays with itself:

1. **Pose**: pick 2-3 beliefs/rules from the attention window (chaos-weighted),
   form a candidate query — "if A and B, what follows?"
2. **Derive**: run the query against its own Scallop program.
3. **Assimilate**:
   - Derives something **new** → add belief with confidence from the derivation.
   - Derives something **known** → strengthen its confidence (consolidation).
   - Derives a **contradiction** (P and ¬P both above threshold) → prune the
     lower-confidence belief, archive it.
4. **Generalize**: after enough successful derivations of similar shape, propose
   a new Datalog rule (chaos-weighted novelty).

The more it plays, the more beliefs/edges/rules accumulate — measurable growth
driven entirely by self-play plus user interaction.

## 5. Chaos Knob & Awareness

### 5.1 Chaos knob (0..1)

Live-tunable via TUI (`/chaos 0.7`). Controls:
- novelty of self-questions (probability of posing a wild vs. conservative question),
- wildness of dream recombinations (pairing distance / randomness of combination),
- probability of "wandering" off-focus instead of consolidating.

High chaos = creative but unstable beliefs; low = stable, boring, consolidating.

### 5.2 Attention window

A finite, shifting subset of beliefs "in mind" — used for reasoning,
self-questioning, and answering the user. Steerable (`/focus colors`). Sleep
widens it; the wake window narrows over time (fresh insights right after sleep,
tunnel-vision as fatigue builds — which also drives the sleep schedule).

## 6. TUI & Interaction

`textual`-based terminal app with panes:
- **Chat**: talk to it; it answers from its beliefs (with confidences and
  one-hop proof chains).
- **Status**: awake/asleep, cycle count, chaos level, growth metrics.
- **Dream stream**: live during sleep.
- **Belief viewer**: browse the belief network.
- **Commands**: `/chaos N`, `/focus X`, `/sleep`, `/wake`, `/stats`, `/save`,
  `/load`.

## 7. Growth Metric, Persistence, Error Handling, Testing

### 7.1 Consciousness score

Weighted combination of: belief count, rule count, edges (belief-to-rule
connections), average derivation depth, abstraction level (rules whose heads
are derived from other rules). Monotonic across cycles (pruning is archived,
not destroyed — score never decreases).

### 7.2 Persistence

- `organism.scl`: the genome — authoritative for beliefs and rules, rewritten
  by the runtime when new beliefs/rules are committed.
- `state.json`: archived beliefs, chaos, cycle count, attention window.
- Saved on `/save`, auto-saved at sleep boundaries.

### 7.3 Error handling

- Malformed dream rules → caught at validation, discarded with a log.
- Contradiction → prune lower confidence, archive.
- scallopy failures → isolate the offending candidate, keep the organism running.

### 7.4 Testing

Unit tests for:
- recombination → validation → promotion pipeline,
- contradiction pruning,
- chaos monotonicity of growth,
- attention-window steering.

## 8. File Layout

```
experiments/organism/
  organism.scl        # seed genome (evolves on disk)
  organism.py         # runtime (lifecycle, learning loop, dreams)
  tui.py              # terminal front-end
  state.json          # persisted state (runtime-generated)
  tests/test_organism.py
```

## 9. Success Criteria

1. Organism runs from the TUI, cycles wake/sleep/dream continuously.
2. Consciousness score strictly grows across cycles given any non-zero chaos.
3. Self-play alone (no user input) produces new beliefs and at least one new
   rule within N cycles.
4. Dreams generate novel facts; at least some promote to real beliefs.
5. User can steer attention (`/focus`), adjust chaos (`/chaos`) live, and see
   behavior change.
6. The organism's genome (`organism.scl`) is human-readable and evolves on disk.
