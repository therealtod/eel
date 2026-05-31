# Addressing the Search Horizon "Dry State" Effect

## Background
The engine currently suffers from a specific form of horizon arbitrage related to newly drawn cards. Since the search tree is deterministic and doesn't model chance nodes for card draws, newly drawn cards are treated as having maximum uncertainty. The static evaluator doesn't assign them direct action value, leading to "dry" states where players hold un-actionable cards. 

Because the engine foresees a forced, heavily penalized discard (e.g., via `bottom_deck_risk_score`) or a strike just beyond the horizon, it assumes the line is disastrous. To avoid this imagined catastrophe, the search pads its lines with inefficient stalling maneuvers, taking minor immediate penalties (like `tempo_cost`) early on to push the forced bad action off the edge of the search horizon.

## Proposed Solutions

### 1. Independent "Future Potential" Heuristic (Recommended)
Currently, the evaluator calculates `clue_demand`, which measures the density of potentially playable or critical cards among untouched cards in players' hands: `demand = Σ max(P(playable), P(critical))`.
However, this demand is primarily used as a *multiplier* on the `clue_token` value (`clue_token_weight * harmonic(n) * (1 + clue_demand_weight * demand)`). If the team has few or no clue tokens (`n=0`), this demand value drops to near zero! This is a structural flaw: when hands are full of fresh cards, they have high expected value, but if the team exhausted clues to draw them, they get zero credit.

**Proposal:** Extract this into an independent leaf term.
Add a new evaluator term: `future_potential_bonus = future_potential_weight * demand`.
This directly rewards paths that cycle the deck and draw new cards, recognizing that unknown cards have a statistical expected value. If `future_potential_weight` is appropriately balanced against `known_playable_weight` (e.g., if a `known_playable` gives 1.0, an unknown card gives `P(playable) * 1.0`), the engine won't artificially penalize itself for cashing in a known playable to draw an unknown card.

### 2. "Dry State" Penalty Forgiveness
If the primary driver of the stalling behavior is the fear of a forced bad action (e.g., `bottom_deck_risk_score`, `efficiency_penalty`, or `strike_penalty`), we could scale these penalties down at the leaf when the team's hands are largely comprised of fresh draws. The leaf evaluator would recognize that its pessimism is an artifact of the search horizon and assume that in reality, the unknown cards would provide safe actions.

### 3. Leaf Rollouts (Monte Carlo Evaluation)
Instead of relying entirely on static evaluation, run a fast rule-based rollout to the end of the game from the leaf state, drawing actual cards sampled from the remaining deck.
* **Pros:** Completely eliminates the horizon effect for draws by resolving the uncertainty.
* **Cons:** Extremely expensive. Alpha-Beta evaluates millions of leaves, and even with Rust's speed, doing rollouts per leaf would require drastically reducing the overall search depth.

### 4. Expected Value (EV) "Phantom" Actions
Inject a "Phantom Draw" logic. While traversing the search tree, instead of drawing an unknown card, the engine could mathematically assume the draw was successful at a rate equal to the remaining deck's density of playables/clueables, granting fractional immediate bonuses.

---

> [!IMPORTANT]
> ## User Review Required
> Please review the proposed solutions above. Option 1 (Future Potential Heuristic) integrates best with the existing alpha-beta evaluator and is computationally cheap. It uses the `DefaultEvaluator`'s existing probability math (`clue_demand`) but properly un-couples it from the clue token bank. 
> 
> Let me know which direction you would like to take, or if you have alternative ideas for how the engine should perceive fresh draws!
