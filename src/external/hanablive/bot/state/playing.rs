use crate::external::hanablive::bot::state::common::CommonState;
use crate::external::hanablive::bot::BotEvent;
use crate::external::hanablive::dto::game_init_data::GameInitData;
use crate::external::hanablive::dto::instruction::game_action_data::GameActionData;
use crate::external::hanablive::dto::instruction::game_action_list::GameAction;
use crate::external::hanablive::dto::instruction::game_action_type::GameActionType;

const PLAYER_ACTIONS: [GameActionType; 4] = [
    GameActionType::Draw,
    GameActionType::Play,
    GameActionType::Discard,
    GameActionType::Clue,
];

pub struct PlayingState {
    pub common_state: CommonState,
    pub table_id: usize,
    pub game_init_data: GameInitData,
    pub sender: tokio::sync::mpsc::UnboundedSender<BotEvent>,
    pub actions_buffer: Vec<GameActionData>,
}

impl PlayingState {
    pub async fn on_game_action(&mut self, game_action: GameAction) -> Result<(), String> {
        if game_action.table_id != self.table_id {
            return Err(format!(
                "Received action for table {} but playing at table {}",
                game_action.table_id, self.table_id
            ));
        }

        self.actions_buffer.push(game_action.action.clone());

        let turn_received = self
            .actions_buffer
            .iter()
            .any(|a| matches!(a, GameActionData::Turn(_)));

        if turn_received {
            self.handle_action_bundle()?;
            self.actions_buffer.clear();
        }

        Ok(())
    }

    fn handle_action_bundle(&mut self) -> Result<(), String> {
        let player_action_count = self
            .actions_buffer
            .iter()
            .filter(|a| PLAYER_ACTIONS.contains(&a.action_type()))
            .count();

        if player_action_count != 1 {
            return Err(format!(
                "Expected exactly one player action, got {}",
                player_action_count
            ));
        }

        let is_strike = self
            .actions_buffer
            .iter()
            .any(|a| matches!(a, GameActionData::Strike(_)));

        let player_action = self
            .actions_buffer
            .iter()
            .find(|a| PLAYER_ACTIONS.contains(&a.action_type()))
            .unwrap();

        match player_action {
            GameActionData::Draw(draw) => {
                tracing::debug!("Draw: player={}, order={}", draw.player_index, draw.order);
            }
            GameActionData::Play(play) => {
                tracing::debug!(
                    "Play: player={}, order={}, suit={}, rank={}, strike={}",
                    play.player_index,
                    play.order,
                    play.suit_index,
                    play.rank,
                    is_strike
                );
            }
            GameActionData::Discard(discard) => {
                tracing::debug!(
                    "Discard: player={}, order={}, suit={}, rank={}",
                    discard.player_index,
                    discard.order,
                    discard.suit_index,
                    discard.rank
                );
            }
            GameActionData::Clue(clue) => {
                tracing::debug!(
                    "Clue: giver={}, target={}, type={}, value={}",
                    clue.giver,
                    clue.target,
                    clue.clue.clue_type,
                    clue.clue.value
                );
            }
            _ => {}
        }

        let current_player_index = self
            .actions_buffer
            .iter()
            .find_map(|a| {
                if let GameActionData::Turn(turn) = a {
                    Some(turn.current_player_index)
                } else {
                    None
                }
            })
            .unwrap_or(0);

        let bot_index = self.game_init_data.our_player_index;
        if current_player_index == bot_index {
            tracing::debug!("It's our turn!");
        }

        Ok(())
    }
}
