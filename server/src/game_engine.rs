use crate::args::ServerArgs;
use derive_getters::Getters;
use shared::resource::StoneSetOperations;
use shared::ZappyError::Network;
use shared::{
    commands::PlayerCmd,
    map::Map,
    player::Player,
    position::{Direction, Position, Side},
    resource::Resource,
    team::Team,
    Egg,
    NetworkError::IsNotConnectedToServer,
    PlayerError, ServerResponse, ZappyError, MAX_COMMANDS,
};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    error::Error,
};

#[derive(Debug, Getters, Clone, PartialEq)]
pub struct GameEngine {
    teams: HashMap<String, Team>,
    players: HashMap<u16, Player>,
    eggs: HashMap<u64, Vec<Egg>>,
    incantation: HashMap<u64, Vec<u16>>,
    map: Map,
    frame: u64,
}

impl GameEngine {
    pub fn from(args: &ServerArgs) -> Result<Self, Box<dyn Error>> {
        let mut map = Map::new(args.width, args.height);
        let teams = args
            .names
            .iter()
            .map(|team_name| {
                let spawn_positions: VecDeque<Position> =
                    (0..args.clients).map(|_| map.random_position()).collect();
                for pos in &spawn_positions {
                    map.field[pos.y][pos.x]
                        .eggs
                        .entry(team_name.clone())
                        .or_insert((0, 0))
                        .1 += 1;
                }
                (
                    team_name.clone(),
                    Team::new(team_name.clone(), spawn_positions),
                )
            })
            .collect();
        Ok(Self {
            incantation: HashMap::new(),
            teams,
            players: HashMap::new(),
            eggs: HashMap::new(),
            map,
            frame: 0,
        })
    }

    fn handle_move(&mut self, player_id: u16, direction: &Direction) {
        let player = self.players.get_mut(&player_id).unwrap();
        let current_x = player.position().x;
        let current_y = player.position().y;

        let (dx, dy) = direction.dx_dy();
        player.set_x((current_x as isize + dx).rem_euclid(*self.map.width() as isize) as usize);
        player.set_y((current_y as isize + dy).rem_euclid(*self.map.height() as isize) as usize);

        self.map.field[current_y][current_x]
            .players
            .remove(player.id());
        self.map.field[player.position().y][player.position().x]
            .players
            .insert(*player.id());
    }

    fn apply_cmd(&mut self, player_id: u16, command: &PlayerCmd) -> Vec<(u16, ServerResponse)> {
        log::debug!("Executing command: {:?} for {}", command, player_id);
        match command {
            PlayerCmd::Left => {
                let player = self.players.get_mut(&player_id).unwrap();
                player.turn(Side::Left);
                vec![(player_id, ServerResponse::Ok)]
            }
            PlayerCmd::Right => {
                let player = self.players.get_mut(&player_id).unwrap();
                player.turn(Side::Right);
                vec![(player_id, ServerResponse::Ok)]
            }
            PlayerCmd::Move => {
                let player_direction = {
                    let player = self.players.get_mut(&player_id).unwrap();
                    player.position().dir
                };
                self.handle_move(player_id, &player_direction);
                vec![(player_id, ServerResponse::Ok)]
            }
            PlayerCmd::Take { resource_name } => {
                let response = Resource::try_from(resource_name.as_str())
                    .map(|resource| {
                        let player = self.players.get_mut(&player_id).unwrap();
                        let cell = &mut self.map.field[player.position().y][player.position().x];
                        if cell.remove_resource(&resource) {
                            player.add_to_inventory(resource);
                            ServerResponse::Ok
                        } else {
                            ServerResponse::Ko
                        }
                    })
                    .unwrap_or(ServerResponse::Ko);
                vec![(player_id, response)]
            }
            PlayerCmd::Put { resource_name } => {
                let response = Resource::try_from(resource_name.as_str())
                    .map(|resource| {
                        let player = self.players.get_mut(&player_id).unwrap();
                        let cell = &mut self.map.field[player.position().y][player.position().x];
                        if player.remove_from_inventory(resource) {
                            cell.add_resource(resource);
                            ServerResponse::Ok
                        } else {
                            ServerResponse::Ko
                        }
                    })
                    .unwrap_or(ServerResponse::Ko);
                vec![(player_id, response)]
            }
            PlayerCmd::See => {
                let player = self.players.get(&player_id).unwrap();
                let pos = *player.position();
                let (player_x, player_y) = (pos.x as isize, pos.y as isize);
                let (width, height) = (self.map_width() as isize, self.map_height() as isize);
                let mut response = Vec::with_capacity((*player.level() as usize + 1).pow(2));
                for line in 0..=(*player.level() as isize) {
                    for idx in -line..=line {
                        let (x, y) = {
                            let (x, y) = match pos.dir {
                                Direction::North => (player_x + idx, player_y - line),
                                Direction::East => (player_x + line, player_y + idx),
                                Direction::South => (player_x - idx, player_y + line),
                                Direction::West => (player_x - line, player_y - idx),
                            };
                            (x.rem_euclid(width) as usize, y.rem_euclid(height) as usize)
                        };
                        let is_same_pos = x == player.position().x && y == player.position().y;
                        let cell = &self.map.field[y][x];
                        let mut cell_response =
                            vec!["player"; cell.players.len() - is_same_pos as usize];
                        cell_response.extend(
                            cell.get_resources_copy()
                                .iter()
                                .map(|resource| resource.as_str())
                                .collect::<Vec<&str>>(),
                        );
                        response.push(cell_response.join(" "));
                    }
                }
                vec![(player_id, ServerResponse::See(response))]
            }
            PlayerCmd::Inventory => {
                let player = self.players.get(&player_id).unwrap();
                let mut inventory = vec![format!(
                    "{} {}",
                    Resource::Nourriture,
                    player.remaining_life()
                )];
                inventory.extend(
                    player
                        .inventory()
                        .iter()
                        .enumerate()
                        .map(|(i, b)| format!("{} {}", Resource::try_from(i).unwrap(), b))
                        .collect::<Vec<String>>(),
                );
                vec![(player_id, ServerResponse::Inventory(inventory))]
            }
            PlayerCmd::Expel => {
                let (target_ids, direction) = {
                    let player = self.players.get(&player_id).unwrap();
                    let ids: Vec<u16> = self.map.field[player.position().y][player.position().x]
                        .players
                        .iter()
                        .filter_map(|&id| if id != player_id { Some(id) } else { None })
                        .collect();
                    (ids, player.position().dir)
                };
                let mut result: Vec<(u16, ServerResponse)> = target_ids
                    .iter()
                    .map(|&id| {
                        self.handle_move(id, &direction);
                        (id, ServerResponse::Movement(direction.opposite()))
                    })
                    .collect();
                result.push((player_id, ServerResponse::Ok));
                result
            }
            PlayerCmd::Broadcast { text } => {
                let sender_pos = self.players.get(&player_id).unwrap().position();
                self.players
                    .keys()
                    .map(|&id| {
                        let resp = if id == player_id {
                            ServerResponse::Ok
                        } else {
                            let receiver_pos = self.players.get(&id).unwrap().position();
                            ServerResponse::Message(
                                self.map.find_broadcast_source(sender_pos, receiver_pos),
                                text.clone(),
                            )
                        };
                        (id, resp)
                    })
                    .collect()
            }
            PlayerCmd::Incantation => {
                let player = self.players.get(&player_id).unwrap();
                let position = player.position();
                if *player.remaining_life() < PlayerCmd::INCANTATION_DURATION {
                    return vec![(player_id, ServerResponse::Ko)];
                }
                let same_lvl_players = self.map.field[position.y][position.x]
                    .players
                    .iter()
                    .filter_map(|&lvl| {
                        let other = self.players.get(&lvl).unwrap();
                        if *other.level() == *player.level()
                            && *other.remaining_life() >= PlayerCmd::INCANTATION_DURATION
                        {
                            Some(*other.id())
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>();
                if same_lvl_players.len() >= player.nxt_lvl_player_cnt_requirements()
                    && self.map.field[position.y][position.x]
                        .stones
                        .reduce_current_from(player.nxt_lvl_stone_requirements())
                {
                    same_lvl_players
                        .iter()
                        .map(|&id| {
                            self.incantation
                                .entry(self.frame + PlayerCmd::INCANTATION_DURATION)
                                .and_modify(|v| v.push(id))
                                .or_insert(vec![id]);
                            self.players.get_mut(&id).unwrap().start_incantation();
                            (id, ServerResponse::IncantationInProgress)
                        })
                        .collect()
                } else {
                    vec![(player_id, ServerResponse::Ko)]
                }
            }
            PlayerCmd::Fork => {
                // TODO: use MAX_PLAYERS to not abuse spamming
                let player = self.players.get(&player_id).unwrap();
                let egg = Egg {
                    team_name: player.team().clone(),
                    position: player.position().clone(),
                };
                self.eggs
                    .entry(self.frame + PlayerCmd::EGG_FETCH_TIME_DELAY)
                    .and_modify(|v| v.push(egg.clone()))
                    .or_insert(vec![egg]);
                self.map.field[player.position().y][player.position().x]
                    .eggs
                    .entry(player.team().clone())
                    .and_modify(|(unhatched, _)| *unhatched += 1)
                    .or_insert((1, 0));
                vec![(player_id, ServerResponse::Ok)]
            }
            PlayerCmd::ConnectNbr => {
                let team_name = self.players.get(&player_id).unwrap().team();
                vec![(
                    player_id,
                    ServerResponse::Value(
                        self.teams
                            .get(team_name)
                            .unwrap()
                            .remaining_members()
                            .to_string(),
                    ),
                )]
            }
        }
    }

    pub fn tick(&mut self, execution_results: &mut Vec<(u16, ServerResponse)>) {
        self.frame += 1;
        let current_frame = self.frame;
        let mut commands_to_process = Vec::new();

        let mut dead_players = HashSet::new();
        for (id, player) in &mut self.players {
            if *player.remaining_life() == 0 {
                log::info!(
                    "Player {} from {} died at ({}, {})",
                    player.id(),
                    player.team(),
                    player.position().x,
                    player.position().y
                );
                execution_results.push((*player.id(), ServerResponse::Mort));
                dead_players.insert(*player.id());
                continue;
            }

            player.decrement_life();

            if !player.commands().is_empty() && current_frame >= *player.next_frame() {
                let command = player.pop_command_from_queue().unwrap();
                player.set_next_frame(current_frame + command.delay());
                commands_to_process.push((*id, command));
            }
        }

        for player_id in dead_players {
            self.remove_player(player_id);
        }

        for (player_id, command) in commands_to_process {
            if *self
                .players
                .get(&player_id)
                .unwrap()
                .is_performing_incantation()
            {
                execution_results.push((player_id, ServerResponse::IncantationInProgress));
            } else {
                execution_results.extend(self.apply_cmd(player_id, &command));
            }
        }

        if let Some(eggs_to_hatch) = self.eggs.remove(&current_frame) {
            for egg in eggs_to_hatch {
                if let (Some((unhatched, hatched)), Some(team)) = (
                    self.map.field[egg.position.y][egg.position.x]
                        .eggs
                        .get_mut(&egg.team_name),
                    self.teams.get_mut(&egg.team_name),
                ) {
                    *unhatched -= 1;
                    *hatched += 1;
                    team.add_next_spawn_position(egg.position);
                    log::info!(
                        "Team {}: hatched egg at ({}, {})!",
                        egg.team_name,
                        egg.position.x,
                        egg.position.y
                    );
                }
            }
        }

        if let Some(players_to_stop_incantation) = self.incantation.remove(&current_frame) {
            for id in players_to_stop_incantation {
                if let Some(player) = self.players.get_mut(&id) {
                    match player.stop_incantation() {
                        Ok(lvl) => execution_results.push((id, ServerResponse::CurrentLevel(lvl))),
                        Err(e) => log::error!("{e}"),
                    }
                }
            }
        }
    }

    pub fn add_player(&mut self, player_id: u16, team_name: String) -> Result<u16, ZappyError> {
        log::debug!("{player_id} wants to join {team_name}");
        let team = self.teams.get_mut(&team_name).ok_or(ZappyError::Player(
            PlayerError::TeamDoesntExist(team_name.clone()),
        ))?;
        let pos = team.add_member(player_id)?;
        let player = Player::new(player_id, team_name.clone(), pos);
        self.map
            .add_player(*player.id(), player.team(), player.position());
        let log_successful_insert = format!(
            "The player with id: {} has successfully joined the \"{}\" team.",
            player.id(),
            player.team()
        );
        self.players.insert(player_id, player);
        log::info!("{log_successful_insert}");
        Ok(team.remaining_members())
    }

    pub fn remove_player(&mut self, player_id: u16) {
        if let Some(player) = self.players.remove(&player_id) {
            log::debug!("Client {player_id} has been removed from the server");
            self.map.remove_player(player.id(), player.position());
            self.teams
                .get_mut(player.team())
                .unwrap()
                .remove_member(player_id);
        }
    }

    pub fn take_command(
        &mut self,
        player_id: &u16,
        cmd: PlayerCmd,
    ) -> Result<Option<ServerResponse>, ZappyError> {
        if let Some(player) = self.players.get_mut(player_id) {
            Ok(if player.commands().len() >= MAX_COMMANDS {
                Some(ServerResponse::ActionQueueIsFull)
            } else {
                player.push_command_to_queue(cmd);
                None
            })
        } else {
            Err(Network(IsNotConnectedToServer(*player_id)))
        }
    }

    pub fn map_width(&self) -> usize {
        *self.map.width()
    }

    pub fn map_height(&self) -> usize {
        *self.map.height()
    }
}

#[cfg(test)]
mod game_engine_tests {
    use super::*;
    use crate::args::ServerArgsBuilder;

    // Common test constants
    const GAME_WIDTH: usize = 3;
    const GAME_HEIGHT: usize = 3;
    const MAX_CLIENTS: u16 = 1;
    const TEST_TEAM_NAME_STR: &str = "Axel";

    fn test_team_name() -> String {
        TEST_TEAM_NAME_STR.to_string()
    }

    fn one_player_game_engine() -> (u16, GameEngine) {
        let player_id = 20;
        let team_name = test_team_name();
        let mut game = default_game_engine();
        game.add_player(player_id, team_name).unwrap();
        (player_id, game)
    }

    fn default_args() -> ServerArgsBuilder {
        ServerArgsBuilder::default()
            .port(8080u16)
            .clients(MAX_CLIENTS)
            .tud(100u16)
            .names(vec![test_team_name()])
            .width(GAME_WIDTH)
            .height(GAME_HEIGHT)
            .to_owned()
    }

    fn default_game_engine() -> GameEngine {
        GameEngine::from(&default_args().build().unwrap()).unwrap()
    }

    mod creation {
        use super::*;

        #[test]
        fn successfully_creates_new_game() {
            // Given
            let args = default_args().build().unwrap();

            // When
            let game = GameEngine::from(&args).unwrap();

            // Then
            // Eggs
            assert!(game.eggs.is_empty(), "New game should have no eggs");
            let (map_unhatched_count, map_hatched_count) =
                game.map.field.iter().flatten().fold((0, 0), |acc, cell| {
                    let (unhatched, hatched) = cell
                        .eggs
                        .get(&test_team_name())
                        .map_or((0, 0), |(u, h)| (*u, *h));
                    (acc.0 + unhatched, acc.1 + hatched)
                });
            assert_eq!(
                map_hatched_count,
                MAX_CLIENTS as usize * game.teams.len(),
                "The unhatched eggs count is max clients per team"
            );
            assert_eq!(
                map_unhatched_count, 0,
                "There is no unhatched eggs on the map"
            );

            // Players
            assert!(game.players.is_empty(), "New game should have no players");
            assert_eq!(
                game.map
                    .field
                    .iter()
                    .flatten()
                    .map(|v| v.players.len())
                    .sum::<usize>(),
                0,
                "There is not players on the game field"
            );

            // Teams
            assert_eq!(
                game.teams.keys().cloned().collect::<Vec<_>>(),
                vec![test_team_name()],
                "Game should have exactly one team named 'axel'"
            );

            // Incantation
            assert!(
                game.incantation.is_empty(),
                "New game should have no active incantations"
            );

            // Game grid
            assert_eq!(
                game.map_width(),
                GAME_WIDTH,
                "Game map width should match configured width"
            );
            assert_eq!(
                game.map_height(),
                GAME_HEIGHT,
                "Game map height should match configured height"
            );
            assert_eq!(
                game.map.field.iter().map(|v| v.len()).sum::<usize>(),
                GAME_HEIGHT * GAME_WIDTH,
                "Game map should contain exactly width * height cells"
            );
            assert!(game.map.field.iter().all(|v| v.len() == GAME_WIDTH));

            // Other
            assert_eq!(game.frame, 0, "Game should start at frame 0");
        }
    }

    mod player_management {
        use super::*;
        use rstest::rstest;

        #[test]
        fn fails_to_add_player_with_invalid_team() {
            // Given
            let player_id = 20;
            let not_existing_team = "Doesn't exist".to_string();
            let mut game = default_game_engine();
            let initial_game_state = game.clone();

            // When
            let result = game.add_player(player_id, not_existing_team.clone());

            // Then
            assert_eq!(
                result,
                Err(ZappyError::Player(PlayerError::TeamDoesntExist(
                    not_existing_team
                )))
            );
            assert_eq!(
                game, initial_game_state,
                "Game should not be changed after failed player insertion"
            );
        }

        #[test]
        fn fails_to_add_player_with_full_team() {
            // Given
            let player_in_team_id = 20;
            let player_to_join_id = 21;
            let team_name = test_team_name();
            let mut game = default_game_engine();
            game.add_player(player_in_team_id, team_name.clone())
                .unwrap();
            let initial_game_state = game.clone();

            // When
            let result = game.add_player(player_to_join_id, team_name.clone());

            // Then
            assert_eq!(
                result,
                Err(ZappyError::Player(PlayerError::NoPlaceAvailable(
                    player_to_join_id,
                    team_name
                ))),
            );
            assert_eq!(
                game, initial_game_state,
                "Game should not be changed after failed player insertion"
            );
        }

        #[rstest]
        #[case(HashMap::from([("Axel".to_string(), 2), ("Anton".to_string(), 5)]))]
        #[case(HashMap::from([("Anton".to_string(), 1), ("Victor".to_string(), 1), ("Axel".to_string(), 1)]
        ))]
        #[case(HashMap::from([("Anton".to_string(), 7), ("Victor".to_string(), 10), ("Axel".to_string(), 25)]
        ))]
        #[case(HashMap::from([("Anton".to_string(), 1)]))]
        fn successfully_adds_player_to_valid_team(#[case] players_to_add: HashMap<String, usize>) {
            // Given
            let mut current_id = 0;
            let max_player_nbr = *players_to_add.values().max().unwrap() as u16;
            let all_players_nbr = players_to_add.values().sum::<usize>();
            let args = default_args()
                .clients(max_player_nbr)
                .names(players_to_add.keys().cloned().collect::<Vec<_>>())
                .build()
                .unwrap();
            let mut game = GameEngine::from(&args).unwrap();
            let mut result: HashMap<String, Vec<(u16, u16)>> = HashMap::new();

            // When
            for (team, player_count) in players_to_add {
                for _ in 0..player_count {
                    let add_player_res = game.add_player(current_id, team.clone()).unwrap();
                    result
                        .entry(team.clone())
                        .and_modify(|v| v.push((current_id, add_player_res)))
                        .or_insert(vec![(current_id, add_player_res)]);
                    current_id += 1;
                }
            }

            // Then
            assert_eq!(
                game.players.len(),
                all_players_nbr,
                "Should add a new player to the players list"
            );
            for (team, players_ids) in &result {
                let expected_response = max_player_nbr;
                for (i, (id, response)) in players_ids.iter().enumerate() {
                    let player = game.players.get(&id).unwrap();

                    assert_eq!(
                        *response,
                        expected_response - (i + 1) as u16,
                        "Should return maximum number of players - 1"
                    );
                    assert!(
                        game.map.field[player.position().y][player.position().x]
                            .players
                            .contains(player.id()),
                        "Should add a new player to the map"
                    );
                    assert_eq!(
                        game.map
                            .field
                            .iter()
                            .flatten()
                            .map(|v| v.players.len())
                            .sum::<usize>(),
                        all_players_nbr,
                        "Should be all players on the game field"
                    );
                    assert!(
                        !player.is_performing_incantation(),
                        "Should not perform incantation"
                    );
                    assert_eq!(
                        player.inventory().iter().sum::<usize>(),
                        0,
                        "Should have empty inventory"
                    );
                    assert_eq!(*player.level(), 1, "Should appear with level 1");
                    assert_eq!(player.commands().len(), 0, "Should have no commands");
                    assert!(
                        game.teams.get_mut(team).unwrap().has_member(&id),
                        "Should add a new player to the team"
                    );
                }
            }
        }
    }

    mod commands_management {
        use super::*;

        #[test]
        fn successfully_takes_command() {
            // Given
            let (player_id, mut game) = one_player_game_engine();
            let player_commands_before = game.players.get(&player_id).unwrap().commands().clone();
            let command = PlayerCmd::Move;

            // When
            let result = game.take_command(&player_id, command.clone());

            // Then
            let player_commands_after = game.players.get(&player_id).unwrap().commands();
            assert!(player_commands_before.is_empty());
            assert_eq!(result, Ok(None));
            assert_eq!(player_commands_after.len(), 1);
            assert_eq!(player_commands_after[0], command);
        }

        #[test]
        fn fails_to_take_player_command_queue_is_full() {
            // Given
            let (player_id, mut game) = one_player_game_engine();
            for _ in 0..MAX_COMMANDS {
                game.take_command(&player_id, PlayerCmd::Move).unwrap();
            }

            // When
            let result_of_inserting_command_in_the_full_queue =
                game.take_command(&player_id, PlayerCmd::Move);

            // Then
            assert_eq!(
                result_of_inserting_command_in_the_full_queue,
                Ok(Some(ServerResponse::ActionQueueIsFull))
            );
        }
    }

    mod commands_execution {
        use super::*;
        use rstest::rstest;
        use std::collections::BTreeMap;
        use Direction::East;
        use Direction::North;
        use Direction::South;
        use Direction::West;

        fn one_player_game_engine_with_player_init_pos(position: Position) -> (u16, GameEngine) {
            let player_id = 20;
            let team_name = test_team_name();
            let mut game = default_game_engine();
            game.teams = HashMap::from([(
                test_team_name(),
                Team::new(test_team_name(), VecDeque::from([position])),
            )]);
            game.map.field[position.y][position.x].eggs =
                BTreeMap::from([(team_name.clone(), (0, 1))]);
            game.add_player(player_id, team_name).unwrap();
            (player_id, game)
        }

        #[rstest]
        // Movement tests - North/South
        #[case(
            Position{ x: 0, y: 0, dir: North },
            Position{ x: 0, y: 2, dir: North },
            PlayerCmd::Move
        )]
        #[case(
            Position{ x: 1, y: 0, dir: North },
            Position{ x: 1, y: 2, dir: North },
            PlayerCmd::Move
        )]
        #[case(
            Position{ x: 2, y: 0, dir: North },
            Position{ x: 2, y: 2, dir: North },
            PlayerCmd::Move
        )]
        #[case(
            Position{ x: 0, y: 2, dir: South },
            Position{ x: 0, y: 0, dir: South },
            PlayerCmd::Move
        )]
        #[case(
            Position{ x: 1, y: 2, dir: South },
            Position{ x: 1, y: 0, dir: South },
            PlayerCmd::Move
        )]
        #[case(
            Position{ x: 2, y: 2, dir: South },
            Position{ x: 2, y: 0, dir: South },
            PlayerCmd::Move
        )]
        // Movement tests - East/West
        #[case(
            Position{ x: 0, y: 0, dir: West },
            Position{ x: 2, y: 0, dir: West },
            PlayerCmd::Move
        )]
        #[case(
            Position{ x: 0, y: 1, dir: West },
            Position{ x: 2, y: 1, dir: West },
            PlayerCmd::Move
        )]
        #[case(
            Position{ x: 0, y: 2, dir: West },
            Position{ x: 2, y: 2, dir: West },
            PlayerCmd::Move
        )]
        #[case(
            Position{ x: 2, y: 0, dir: East },
            Position{ x: 0, y: 0, dir: East },
            PlayerCmd::Move
        )]
        #[case(
            Position{ x: 2, y: 1, dir: East },
            Position{ x: 0, y: 1, dir: East },
            PlayerCmd::Move
        )]
        #[case(
            Position{ x: 2, y: 2, dir: East },
            Position{ x: 0, y: 2, dir: East },
            PlayerCmd::Move
        )]
        // Rotation tests - Left
        #[case(
            Position{ x: 1, y: 1, dir: North },
            Position{ x: 1, y: 1, dir: West },
            PlayerCmd::Left
        )]
        #[case(
            Position{ x: 1, y: 1, dir: West },
            Position{ x: 1, y: 1, dir: South },
            PlayerCmd::Left
        )]
        #[case(
            Position{ x: 1, y: 1, dir: South },
            Position{ x: 1, y: 1, dir: East },
            PlayerCmd::Left
        )]
        #[case(
            Position{ x: 1, y: 1, dir: East },
            Position{ x: 1, y: 1, dir: North },
            PlayerCmd::Left
        )]
        // Rotation tests - Right
        #[case(
            Position{ x: 1, y: 1, dir: North },
            Position{ x: 1, y: 1, dir: East },
            PlayerCmd::Right
        )]
        #[case(
            Position{ x: 1, y: 1, dir: East },
            Position{ x: 1, y: 1, dir: South },
            PlayerCmd::Right
        )]
        #[case(
            Position{ x: 1, y: 1, dir: South },
            Position{ x: 1, y: 1, dir: West },
            PlayerCmd::Right
        )]
        #[case(
            Position{ x: 1, y: 1, dir: West },
            Position{ x: 1, y: 1, dir: North },
            PlayerCmd::Right
        )]
        fn test_player_movement_and_rotation(
            #[case] start: Position,
            #[case] expected: Position,
            #[case] command: PlayerCmd,
        ) {
            // Given
            let (player_id, mut game) = one_player_game_engine_with_player_init_pos(start);
            let mut execution_results_buffer = Vec::new();

            // Set initial position
            game.players
                .get_mut(&player_id)
                .unwrap()
                .set_position(start);

            // When
            game.take_command(&player_id, command.clone()).unwrap();

            // Execute command
            for _ in 0..command.delay() {
                game.tick(&mut execution_results_buffer)
            }

            // Then
            let player = game.players.get(&player_id).unwrap();
            let new_position = player.position();

            // Verify player is at new position
            assert!(
                game.map.field[new_position.y][new_position.x]
                    .players
                    .contains(&player_id),
                "Player should be present at new position"
            );

            // Verify player is not at old position (only for movement)
            if (start.x, start.y) != (expected.x, expected.y) {
                assert!(
                    !game.map.field[start.y][start.x]
                        .players
                        .contains(&player_id),
                    "Player should not be present at old position"
                );
            }

            // Verify game state
            assert_eq!(
                game.frame,
                command.delay(),
                "Game frame should match command delay"
            );
            assert_eq!(
                *new_position, expected,
                "Player position should match expected"
            );

            // Verify response
            assert_eq!(
                execution_results_buffer,
                vec![(player_id, ServerResponse::Ok)],
                "Should receive OK response"
            );
        }
    }
}
