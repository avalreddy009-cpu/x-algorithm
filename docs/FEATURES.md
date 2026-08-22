# For You features (easy English)

This is a simple list of **what the feed already does**, plus **helpful extras** that would make it better for viewers.

Nothing here changes live X by itself. Weights and filters live in `home-mixer/`. Safety rules live in `visibility-filtering/`.

---

## A. Existing features

### 1. Finding posts

| Feature | In plain English |
| --- | --- |
| Posts from people you follow (Thunder) | Recent posts from accounts you follow |
| Suggested posts (Phoenix retrieval) | Posts from people you do not follow, chosen because they look similar to what you like |
| Similar-cluster posts (SimClusters) | Posts from groups of accounts that get similar engagement |
| Topic posts | Posts tied to topics you follow or that match a topic request |
| Cached posts | Reuse recent ranked posts so the feed loads faster |
| Following reverse-chron | Following tab can show newest-first instead of ranked |
| Night-owl following | Extra following posts for late-night browsing |
| Popular topics | A source of trending / popular topic posts |
| Seed candidates | Starting posts used to help retrieval |

### 2. Extra items mixed into the feed

| Feature | In plain English |
| --- | --- |
| Ads | Paid posts placed next to organic posts |
| Who to Follow | Account suggestions |
| Prompts | In-feed prompts (for example, feedback or setup) |
| Push-to-home | A post from a notification can be placed on Home |
| Surveys | Occasional feed surveys |
| Jetfuel frames | Special frame / creative units |

### 3. What the model knows about you

| Feature | In plain English |
| --- | --- |
| Recent actions | Likes, replies, dwell, and other things you just did |
| Follows, blocks, mutes | Who you want and who you do not want |
| Muted keywords | Hide posts that contain words you muted |
| Already seen / already served | Do not keep showing the same post |
| Subscriptions | Subscriber-only posts you can or cannot open |
| Mutual follows | Whether someone follows you back |
| Topics and starter packs | Topics and packs you follow |
| Demographics / country | Used for legal and safety rules (age, location) |

### 4. Ranking (order of posts)

The model guesses what **you** might do, then adds those guesses into one score.

**Positive guesses (raise the post)**

- Like (favorite)
- Reply
- Repost
- Quote
- Share, share via DM, copy link
- Click the post, profile, or link
- Expand a photo / open a video
- Watch video well (quality view)
- Dwell (you stayed and read)
- Dwell time and click-dwell time
- Follow the author
- Quoted-post click / quoted video view
- “Unexplored post” boost (help posts that have not been shown much)

**Negative guesses (lower the post)**

- Not interested
- Mute author
- Block author
- Report
- Not dwelled (you skipped past it)

**After the score**

| Feature | In plain English |
| --- | --- |
| Author diversity | If one person already appeared, their next posts go down |
| Out-of-network discount | Suggestions from strangers score a bit lower |
| Reply / repost discount | Some replies and reposts from follows are also cut |
| New-author boost | Small / new accounts can be lifted so they get a chance |
| Bidirectional reply boost | Extra weight on replies to people who follow you back |
| Bidirectional dwell boost | Same idea for reading time (tested; not always on) |
| Click-dwell low-like penalty | Long look but almost no likes can be treated as weaker |
| Diversity rerank (VMRanker) | Shuffle a little so neighbors are not all the same |
| Top-K | Keep only the highest scoring posts |

### 5. Filters (remove posts)

**Before scoring**

| Feature | In plain English |
| --- | --- |
| Drop duplicates | Same post from two sources → keep one |
| Missing data | Drop posts that failed to load |
| Age (48 hours) | Drop old posts |
| Self posts | Do not put your own posts in For You |
| Stranger replies / reposts | Do not recommend replies/reposts from people you do not follow |
| Adult SimClusters | Hide adult-flagged cluster posts unless you follow the author |
| Repeated reposts | Same original post, many reposts → keep fewer |
| Subscriptions | Hide paid posts you cannot see |
| Already seen / served | Hide posts you already got |
| Muted keywords | Hide matching text |
| Blocks and mutes | Hide those authors |
| Video off | Hide video when the client asked for no video |
| Topics | Keep only requested topics; drop excluded ones |
| New-user bar | For new accounts, hide weak stranger posts |
| Holdout | Hold back a slice of posts for experiments |
| Brazil 2026 election | For viewers in that legal setting, hide listed reported accounts unless you follow them |

**After ranking**

| Feature | In plain English |
| --- | --- |
| Visibility filter | Safety system says show, warn, or hide |
| Ancillary hide | If a quoted / parent / reposted post is hidden, hide this one too |
| Conversation dedup | Do not show many branches of the same thread |
| Ad-adjacent served | Avoid certain posts sitting next to ads |

### 6. Safety and labels (can this be shown?)

| Feature | In plain English |
| --- | --- |
| Allow / interstitial / drop | Show normally, show behind a tap-through warning, or hide |
| Stronger rules for suggestions | Spam can be hidden as a recommendation but still shown to followers |
| Blocks, mutes, protected, suspended | Respect your graph and account state |
| Adult / graphic media | Age gates and warning screens |
| Legal takedowns, DMCA, geo-blocked media | Hide when the law or rights require it |
| Account quality models | Agatha (blocks/reports vs likes), BDSM (fake/abuse patterns), user-cred (graph reputation) |
| Media models | Images and video for adult, violence, hateful symbols |
| Grox classifiers | Text/media categories such as spam |
| Botmaker / Scarecrow rules | If this event happens and these conditions match, apply a label |
| Abuse enforcement | From model scores: label, challenge, or suspend |
| Under the Hood | You can see aggregate labels that affect visibility |

### 7. Transparency and experiments

| Feature | In plain English |
| --- | --- |
| Open weights | Production-like default weights are in `home-mixer/params/param.rs` |
| Feature switches | Stages can be turned on/off for tests |
| Phoenix training code | You can train a small ranking model locally |
| Synthetic data | Fake data so training can run without real user logs |

---

## B. Helpful features to add

These are **not in the repo today** as first-class For You controls. They would help viewers, creators, and people who audit the algorithm.

### Viewer control (highest value)

| Proposed feature | Why it helps | How it could fit |
| --- | --- | --- |
| **Interest sliders** | Let people ask for more news, more friends, less video, more replies | Multiply Phoenix action weights per viewer (reply vs video-open vs OON discount) |
| **“More like this” / “less like this” that lasts** | One “not interested” is easy to undo by the next session | Store a durable topic/author embedding penalty in the viewer sequence |
| **Following-only For You mode** | Some people want friends, not suggestions | Set Phoenix/SimClusters sources off; keep Thunder only |
| **Snooze an author for 7 days** | Mute is too permanent; skip is too weak | Time-boxed author filter next to mutes |
| **Hide topics, not just keywords** | Keywords miss slang and images | Use topic IDs already on posts (`TopicIdsFilter`) as a user deny-list |
| **Language lock** | Stop mixing languages you do not read | Filter on `language_code_hydrator` unless the user follows the author |
| **Reply-guy / quote-dunk downrank** | Long dunk threads feel noisy | Extra OON discount on quote/reply posts |
| **Seen-enough on a story** | After many posts on one event, show other things | Cluster posts by semantic id; decay later ones like author diversity |

### Health and quality

| Proposed feature | Why it helps | How it could fit |
| --- | --- | --- |
| **Rage-bait detector in ranking** | High dwell + high report + low like is often bait | New negative head, or use the existing click-dwell low-like penalty more strongly |
| **Creator-to-viewer match, not just viral** | Viral posts drown smaller authors who actually fit you | Stronger new-author boost + weaker global engagement counts in hydration |
| **Repost-of-repost collapse** | The same clip appears five times | Extend retweet dedup to near-duplicate media embeddings (CLIP) |
| **Conversation completeness** | Ranked replies without parent context confuse people | Prefer original posts; keep `OONRetweetReplyFilter` strict |
| **Time-of-day freshness** | Breaking news dies behind 48h + engagement | Recency multiplier on score (newer → slightly higher) |
| **Follow-graph trust** | Spam rings game likes | Downrank authors with low user-cred when OON |
| **Media-first vs text-first** | Some users hate video autoplay piles | Viewer-level video-open / VQV weight |

### Trust and fairness

| Proposed feature | Why it helps | How it could fit |
| --- | --- | --- |
| **Why am I seeing this?** | People distrust a black box | Log the top 3 score terms (reply vs dwell vs OON cut) onto the client debug payload |
| **Label → feed effect map** | Under the Hood shows labels; it is hard to see *feed* impact | Table: label name → VF drop vs interstitial vs ranking-only |
| **Election / civic freshness without hiding news** | Legal filters can also hide useful discussion | Keep follow-exception (already used in Brazil filter); add civic topic boost from *followed* news accounts |
| **Appeals-friendly drops** | Users think they are shadowbanned | Surface VF drop reason in Under the Hood when it is an automated label |

### Builder / research (this open-source repo)

| Proposed feature | Why it helps |
| --- | --- |
| **Score explainer CLI** | Given fake Phoenix probs, print the weighted sum so people stop misreading report vs like weights |
| **Filter playground** | Run one post + one viewer through pre-scoring filters in a unit test harness |
| **Weight recipe presets** | Named sets: “friends first”, “news first”, “discovery first” as param bundles |
| **Easier Phoenix quickstart on CPU** | More people can try training without a big GPU cluster |

---

## C. Simple “good next” set

If you only add a few, start here:

The following are **implemented in this fork** (Home Mixer params + Phoenix training helpers):

- Friends-first OON discount (`EnableFriendsFirstForYou`, default on, factor 0.55)
- Recency boost, quality-at-equal-impressions tax, reply-farm gate, reciprocal conversation boost, SID diversity
- Optional language lock (`EnableLanguageLock`, default off)
- `score_explain` on ranked posts; CLI `phoenix/reference/score_explainer.py`
- Phoenix `ethical_log1p_impressions` on the nano ranking config; `xrex.data.recsys.ethical_weights`

1. **Durable less-like-this** (author + topic) — still a follow-up (needs a persisted negative bag)
2. **Language lock** — implemented; turn on the param
3. **Recency boost** — implemented
4. **Near-duplicate SID collapse** — implemented as score decay
5. **Why this post** — `score_explain`
6. **Friends-first preset** — implemented

Those items sit on existing sources, weights, language hydration, and semantic IDs.

---

## D. What “better” should not do

- Do not hide safety policy inside secret Phoenix weights. Keep **rank vs hide** separate so people can audit it.
- Do not treat raw like/report **counts** as the score. The model scores **your chance** of acting.
- Do not recommend replies and reposts from strangers by default. That is already a spam magnet.
