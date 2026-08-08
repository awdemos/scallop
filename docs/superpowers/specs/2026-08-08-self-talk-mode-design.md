# Self-Talk Mode — Design

Date: 2026-08-08
Status: approved

## Summary

A toggleable TUI mode (`/self-talk`) that lets the organism speak to itself:
when enabled, the organism asks itself a question about its own beliefs and
answers it, in the LLM's first-person voice. The Q&A is recorded into the
persistent chat log so it becomes part of the organism's memory and future
narration context.

## Behavior

- `/self-talk` toggles the mode; the dreams pane shows `self-talk on` /
  `self-talk off`.
- Toggle ON while awake fires one Q&A episode immediately, and the existing
  15s narration interval then produces self-talk Q&A episodes instead of
  plain thoughts for as long as the mode is on.
- Toggle OFF returns narration to normal thoughts.
- There is **no new timer** — only the existing 15s interval and the
  immediate fire on toggle trigger episodes.

## State routing

| State  | Mode off                       | Mode on                                  |
|--------|--------------------------------|------------------------------------------|
| wake   | normal thought narration       | self-talk Q&A (ask + answer, chat_log)   |
| sleep  | dream narration (dream shown)  | dream narration — **no self-talk**       |
| dead   | **full silence**               | **full silence**                         |

### Full silence when dead (new behavior)

- The 15s narration timer is skipped while dead.
- `/think` / ctrl+t shows a silence message instead of narrating.
- Direct chat gets no LLM reply; the dreams pane shows
  `it is silent — the organism has faded. /revive to bring it back.`
  (the user's line is still recorded to the chat log).
- `/revive` restores speech.
- The death transition itself still shows the one-line
  `the organism has faded.` event in the dreams pane — a system message,
  not LLM talk.

## Episode flow (background thread)

1. `narration.self_ask(org)` — LLM self-framing prompt ("ask yourself one
   question about what you believe"), falling back to a deterministic
   template question drawn from a random belief.
2. `narration.self_answer(org, question)` — LLM self-framing answer prompt,
   falling back to a deterministic answer.
3. Both lines are recorded via `record_chat("org", ...)` — persisted in
   state.json and included in future narration context (chat log cap 24).
4. The dreams pane shows `self: {question}` then `self: {answer}`.

## Code changes

| File                    | Change                                                                 |
|-------------------------|------------------------------------------------------------------------|
| `narration.py`          | `self_ask(org)` + `self_answer(org, question)` + `fallback_self_ask` / `fallback_self_answer` |
| `tui.py`                | `_self_talk_on` flag; `/self-talk` branch in `handle_command`; `_maybe_narrate` state routing; dead guards in `_maybe_narrate`, `_maybe_respond`, `action_think_now`; silence message in `handle_chat` |
| `tui_commands.py`       | Register `("/self-talk", "/self-talk", "let the organism speak to itself")` |
| tests                   | narration fallback shapes; `/self-talk` completion; toggle + routing (monkeypatched) |

## Verification

- `PYTHONPATH=etc/scallopy .env/bin/python -m pytest experiments/organism/tests/ -q`
- `ruff check experiments/organism/`
