# For You: durable habits that survive disclosure

Product job: rank a mixed feed (people I follow + suggestions) so I get a
useful picture of my people and the world **without reading everything**.

User goals (treat as hypotheses to **ask**, then measure): stay informed,
hear from friends, find new voices, leave without waste or regret.

Constraint: ranking and visibility are separate; Under the Hood plus open
weights mean **disclosure is a product surface**, not a threat.

Research notes (Octen web search, 2026-08-22): Self-Determination Theory
(autonomy, competence, relatedness); Fogg Tiny Habits (anchor a small action
to a routine that already exists); revealed preference that people install
extensions to **default Following** and to **replace infinite scroll with
Show more**; DSA/OECD framing of dark patterns as steering people into
choices they would not make if the UI were honest; recommender HCI that
trust rises when users can **see and edit** why an item was shown.

---

## A) Sustainable levers (pass the manipulation test)

Prioritized. Each mechanic would still work — often *better* — if X posted
exactly how it works in this repository.

### A1. Onboarding aha: “My people are here, plus one explained extra”

**Need.** Relief (the firehose is handled) and competence (I did not miss
the people I chose). Not a fake viral hit.

**Mechanic.**

- First 3–7 days: **quota** For You so in-network (Thunder) is the majority
  of the first screen. Phoenix/SimClusters get a **single labeled slot**.
- That slot must have a one-line reason the model can actually compute:
  “Because you follow @A” / “Because you linger on this topic” / “Because
  you reply to threads like this.” No unlabeled stranger pile.
- Skip if the user has almost no follows: aha is **follow 5 people whose
  posts then appear**, not a TikTok-style clip. The Following-tab extensions
  people already install are the demand signal.

**User-benefit vs vanity.** Pulse after first session: “Did you see people
you follow?” and “Was the extra post worth it?” Day-7: those suggested
authors still followed. **Do not** use first-session dwell or video VQV as
the aha KPI (that selects bait).

**Disclosure test.** “We show friends first, then one suggestion we can
explain.” Still works. **Pass.**

### A2. Habit: tiny catch-up on a trigger they already have

**Need.** Relief — coffee, commute, queue. Fogg: attach a **tiny** action to
an existing prompt; do not invent a new ping.

**Mechanic.**

- Optional ritual: “After I sit down with coffee, I open X for a **caught-up
  briefing**” — top N from follows, then a hard **You’re caught up with
  people you follow** row and a **Show more / Discover** choice.
- Default N is small (e.g. 10–20). Discover is opt-in per session, not the
  silent remainder of an infinite list.
- No extra notifications to create the ritual. The trigger is the kettle,
  not a badge.

**User-benefit vs vanity.** Rate of reaching caught-up **and leaving**
without a regret flag. Goal item: “I heard from my people.” Sessions may get
**shorter**. That is success if satisfaction is up.

**Disclosure test.** “We end the friends batch on purpose so you can stop.”
Users already install Show more extensions. **Pass.**

### A3. Autonomy: controls that move the same knobs this repo publishes

**Need.** Autonomy (SDT). Habits last when the person is the author of the
behavior.

**Mechanic.**

- Friends ↔ Discover slider mapped to `OonWeightFactor` / source quotas.
- “More conversation / less video” mapped to `ReplyWeight` vs `VideoOpenWeight`
  / `VqvWeight`.
- Durable less-like-this (author + topic), snooze 7 days, language lock,
  Following-only For You.
- Why this post: top score terms (reply vs dwell vs OON cut). Aligns with
  scrutable recommender UIs (user-model editing, not a mysterious blob).

**User-benefit vs vanity.** Control used more than once **and** mute/block
rate down (the product listened). High mute/block with unused controls =
theater.

**Disclosure test.** “This slider is the OON discount you can read in
`param.rs`.” That is the point of open-sourcing weights. **Pass.**

### A4. Relatedness: pay reciprocal talk, not dunk volume

**Need.** Connection — being answered, not performing for a crowd.

**Mechanic.**

- Keep bidirectional reply boost; add a **reciprocal** head (author replies
  to the viewer) gated to in-network / similar-size authors.
- Soft reply-farm gate (unique repliers, quote-dunk + low residual dwell).
- Notifications: someone you care about replied **to you**. Not: a stranger
  dunked on a viral post.

**User-benefit vs vanity.** Author→viewer replies per 1k impressions,
conversation depth ≥2, **low** regret after reply sessions. Total reply
count is vanity (farms it).

**Disclosure test.** “We boost conversations with people who follow you
back, not pile-ons.” Users already asked for this in public. **Pass.**

### A5. Identity: the feed is the graph they built

**Need.** Identity — “this is my corner of X.”

**Mechanic.**

- Onboarding = people + 2–3 topics they pick; For You stays follow-heavy
  until that graph is dense.
- Suggestions visually distinct from follows (never dress a rec as a follow).
- Friends-only mode as a first-class tab behavior, not a buried switch.

**User-benefit vs vanity.** Share of impressions from accounts they follow;
user can explain their feed to a friend in one sentence.

**Disclosure test.** “Suggestions stay a minority until you opt into
discovery.” **Pass.**

### A6. Curiosity: a named Surprise slot, not stealth OON

**Need.** Curiosity, without hijacking catch-up.

**Mechanic.**

- At most one **Surprise** module per briefing, from an explore quota /
  low-impression index, with Skip surprise for a week.
- Never fill the whole first screen with unlabelled strangers (the 2023
  For You complaint pattern).

**User-benefit vs vanity.** Intentional taps on Surprise; skip is a healthy
signal, not a failure. Day-7 follows from Surprise vs from unlabeled OON.

**Disclosure test.** “Slot 7 is explore from a separate index.” Interesting
because it is optional. **Pass.**

### A7. Network effect: denser *your* graph, not FOMO about the globe

**Need.** Relatedness that improves when **your people** show up.

**Mechanic.**

- Mutual-follow boost, conversation completeness (don’t rank orphan replies),
  caught-up with follows.
- Invite/value copy: “More of your people here → better catch-up,” never
  “You’re missing what everyone saw.”

**User-benefit vs vanity.** Two-way threads with existing follows. The
product should work for a user whose **stranger** network is small.

**Disclosure test.** Classic same-side network effect (phones). **Pass.**

### A8. Retention: last visit paid the job, so the next open is obvious

**Need.** Competence + relief.

**Mechanic.**

- Resume / don’t restack the same outrage; recency for followed news;
  durable NI; honest “you’re caught up.”
- Retention email/push **only** for in-network events the user would
  recognize as their job (reply to you, people you follow posted).

**User-benefit vs vanity.** Return after a **short satisfied** session beats
return after a long regretted one. Track “worth my time?” vs minutes.

**Disclosure test.** “We don’t rerun bait to haul you back.” Trust goes up.
**Pass.**

---

## B) Tricks to avoid (fail disclosure; brand and regulatory risk)

Do not ship these even if they lift DAU. DSA/OECD language: interfaces that
**steer, deceive, or coerce**. Infinite scroll without a stop, maze-like
controls, and fake urgency are the feed-shaped versions.

| Trick | Apparent win | Why it fails the test |
| --- | --- | --- |
| Infinite For You with **no caught-up / Show more** | Minutes | “We hide the end so you continue.” Users already fight this with extensions. |
| Variable-ratio viral jackpots as the aha | Day-1 dwell | Only works if unexplained; leftover is addiction, not a habit. |
| Badge/ping for low-value global events | Opens | “We ping so you open, not because someone needs you.” Anxiety, not relatedness. |
| Streaks, freezes, “you’ll lose progress” | Daily open | Manufactured trigger. Disclosure = we punish absence. |
| Recs styled as follows / unlabeled OON majority | Distribution | Hidden attention cost; people install Following-default extensions. |
| Hide mute, less-like-this, Following-only | Accidental consumption | Autonomy theater. Maze navigation is a named dark pattern. |
| Autoplay + dwell as the success metric | Time-on-site | “We count staring as love.” Session regret. |
| “Others are viewing” / scarcity of posts | Now-or-never | FOMO network effect, not value network effect. |
| Ranking that **requires** users not to understand weights | Bait | Contradicts open `param.rs` and Under the Hood. |
| Fake urgency, confirmshaming to turn Discover back on | Suggestion fill | Coercion. Cut. |

**Red-line rule.** If publishing the mechanic in this repo would make a
reasonable user feel tricked, it is not a growth lever.

---

## Scoreboard (so A is not secretly optimized for B)

| Goal they stated | Success (user) | Vanity (cut if it fights success) |
| --- | --- | --- |
| Hear from friends | Caught-up reached; in-network share | Session length |
| Stay informed | Recency on followed news; low “I missed it” | Outrage dwell |
| Find new voices | Surprise follows still followed at d7 | Raw OON impressions |
| Avoid regret | Worth-it pulse; leave after stop row | Opens per day, VQV, unlabelled dwell |

Pair tests: if minutes ↑ and worth-it ↓, you are extracting. If opens ↑ and
mute/block ↑, you are inducing anxiety. If suggestions ↑ and d7 follows ↓,
discovery is fake.
