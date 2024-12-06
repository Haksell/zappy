use derive_getters::Getters;
use shared::{
    commands::PlayerCmd,
    map::Map,
    player::Player,
    position::{Direction, Position, Side},
    resource::{Resource, StoneSetOperations},
    team::Team,
    Egg,
    NetworkError::IsNotConnectedToServer,
    PlayerError, ServerResponse, ZappyError,
    ZappyError::Network,
    MAX_COMMANDS,
};
use std::collections::{BTreeMap, HashSet, VecDeque};

use crate::args::ServerArgs;

#[derive(Debug, Getters, Clone, PartialEq)]
pub struct GameEngine {
    teams: BTreeMap<String, Team>,
    players: BTreeMap<u16, Player>,
    eggs: BTreeMap<u64, Vec<Egg>>,
    incantation: BTreeMap<u64, Vec<u16>>,
    map: Map,
    frame: u64,
}

impl GameEngine {
    pub fn new(args: &ServerArgs) -> Self {
        let mut map = Map::empty(args.width, args.height);
        map.generate_resources();
        let teams = args
            .names
            .clone()
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
        Self {
            incantation: BTreeMap::new(),
            teams,
            players: BTreeMap::new(),
            eggs: BTreeMap::new(),
            map,
            frame: 0,
        }
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
            PlayerCmd::Take(resource_name) => {
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
            PlayerCmd::Put(resource_name) => {
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
            PlayerCmd::Broadcast(text) => {
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

            player.decrease_life();

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
        //TODO: implement and uncomment
        //self.map.generate_resources();
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
    use crate::args::{ServerArgs, ServerArgsBuilder};

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

    fn player_lvl_up(player: &mut Player, level: u8) {
        for _ in 1..level {
            player.start_incantation();
            player.stop_incantation().unwrap();
        }
    }

    fn player_set_hp(player: &mut Player, value: u64) {
        while *player.remaining_life() < value {
            player.add_to_inventory(Resource::Nourriture)
        }
        while *player.remaining_life() > value {
            player.decrease_life()
        }
    }

    fn default_game_engine() -> GameEngine {
        GameEngine::new(&default_args().build().unwrap())
    }

    fn resources_sum_on_other_cell(player_id: &u16, game: &GameEngine) -> usize {
        game.map
            .field
            .iter()
            .flatten()
            .filter(|v| !v.players.contains(&player_id))
            .map(|c| c.stones.iter().sum::<usize>() + c.nourriture)
            .sum::<usize>()
    }

    mod creation {
        use super::*;

        #[test]
        fn successfully_creates_new_game() {
            // Given
            let args = default_args().build().unwrap();

            // When
            let game = GameEngine::new(&args);

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
        #[case(BTreeMap::from([("Axel".to_string(), 2), ("Anton".to_string(), 5)]))]
        #[case(BTreeMap::from([("Anton".to_string(), 1), ("Victor".to_string(), 1), ("Axel".to_string(), 1)]))]
        #[case(BTreeMap::from([("Anton".to_string(), 7), ("Victor".to_string(), 10), ("Axel".to_string(), 25)]))]
        #[case(BTreeMap::from([("Anton".to_string(), 1)]))]
        fn successfully_adds_player_to_valid_team(#[case] players_to_add: BTreeMap<String, usize>) {
            // Given
            let mut current_id = 0;
            let max_player_nbr = *players_to_add.values().max().unwrap() as u16;
            let all_players_nbr = players_to_add.values().sum::<usize>();
            let args = default_args()
                .clients(max_player_nbr)
                .names(players_to_add.keys().cloned().collect::<Vec<_>>())
                .build()
                .unwrap();
            let mut game = GameEngine::new(&args);
            let mut result: BTreeMap<String, Vec<(u16, u16)>> = BTreeMap::new();

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
        use shared::resource::Stone::*;
        use shared::resource::StoneSet;
        use shared::LIFE_TICKS;
        use std::collections::BTreeMap;
        use Direction::*;
        use Resource::*;

        fn game_engine_with(
            positions: &Vec<Position>,
            resources: Option<&Vec<((usize, usize), Resource)>>,
        ) -> (Vec<u16>, GameEngine) {
            let team_name = test_team_name();
            let mut game = default_game_engine();
            game.map = Map::empty(GAME_WIDTH, GAME_HEIGHT);
            let mut res = Vec::new();
            game.teams = BTreeMap::from([(
                test_team_name(),
                Team::new(test_team_name(), VecDeque::from(positions.clone())),
            )]);
            if let Some(resources) = resources {
                for ((x, y), res) in resources {
                    game.map.field[*y][*x].add_resource(*res)
                }
            }
            for (i, pos) in positions.iter().enumerate() {
                game.map.field[pos.y][pos.x].eggs = BTreeMap::from([(team_name.clone(), (0, 1))]);
                game.add_player(i as u16, team_name.clone()).unwrap();
                res.push(i as u16);
            }
            (res, game)
        }

        #[rstest]
        // Movement tests - North/South
        #[case(Position{ x: 0, y: 0, dir: North }, Position{ x: 0, y: 2, dir: North }, PlayerCmd::Move)]
        #[case(Position{ x: 1, y: 0, dir: North }, Position{ x: 1, y: 2, dir: North }, PlayerCmd::Move)]
        #[case(Position{ x: 2, y: 0, dir: North }, Position{ x: 2, y: 2, dir: North }, PlayerCmd::Move)]
        #[case(Position{ x: 0, y: 2, dir: South }, Position{ x: 0, y: 0, dir: South }, PlayerCmd::Move)]
        #[case(Position{ x: 1, y: 2, dir: South }, Position{ x: 1, y: 0, dir: South }, PlayerCmd::Move)]
        #[case(Position{ x: 2, y: 2, dir: South }, Position{ x: 2, y: 0, dir: South }, PlayerCmd::Move)]
        // Movement tests - East/West
        #[case(Position{ x: 0, y: 0, dir: West }, Position{ x: 2, y: 0, dir: West }, PlayerCmd::Move)]
        #[case(Position{ x: 0, y: 1, dir: West }, Position{ x: 2, y: 1, dir: West }, PlayerCmd::Move)]
        #[case(Position{ x: 0, y: 2, dir: West }, Position{ x: 2, y: 2, dir: West }, PlayerCmd::Move)]
        #[case(Position{ x: 2, y: 0, dir: East }, Position{ x: 0, y: 0, dir: East }, PlayerCmd::Move)]
        #[case(Position{ x: 2, y: 1, dir: East }, Position{ x: 0, y: 1, dir: East }, PlayerCmd::Move)]
        #[case(Position{ x: 2, y: 2, dir: East }, Position{ x: 0, y: 2, dir: East }, PlayerCmd::Move)]
        // Rotation tests - Left
        #[case(Position{ x: 1, y: 1, dir: North }, Position{ x: 1, y: 1, dir: West }, PlayerCmd::Left)]
        #[case(Position{ x: 1, y: 1, dir: West }, Position{ x: 1, y: 1, dir: South }, PlayerCmd::Left)]
        #[case(Position{ x: 1, y: 1, dir: South }, Position{ x: 1, y: 1, dir: East }, PlayerCmd::Left)]
        #[case(Position{ x: 1, y: 1, dir: East }, Position{ x: 1, y: 1, dir: North }, PlayerCmd::Left)]
        // Rotation tests - Right
        #[case(Position{ x: 1, y: 1, dir: North }, Position{ x: 1, y: 1, dir: East }, PlayerCmd::Right)]
        #[case(Position{ x: 1, y: 1, dir: East }, Position{ x: 1, y: 1, dir: South }, PlayerCmd::Right)]
        #[case(Position{ x: 1, y: 1, dir: South }, Position{ x: 1, y: 1, dir: West }, PlayerCmd::Right)]
        #[case(Position{ x: 1, y: 1, dir: West }, Position{ x: 1, y: 1, dir: North }, PlayerCmd::Right)]
        fn successfully_applies_movement_and_rotation_commands(
            #[case] start: Position,
            #[case] expected: Position,
            #[case] command: PlayerCmd,
        ) {
            // Given
            let (player_ids, mut game) = game_engine_with(&vec![start], None);
            let player_id = player_ids[0];
            let mut execution_results_buffer = Vec::new();
            game.take_command(&player_id, command.clone()).unwrap();

            // When
            for _ in 0..command.delay() {
                game.tick(&mut execution_results_buffer)
            }

            // Then
            let player = game.players.get(&player_id).unwrap();
            let new_position = player.position();

            assert!(
                game.map.field[new_position.y][new_position.x]
                    .players
                    .contains(&player_id),
                "Player should be present at new position"
            );

            assert_eq!(
                game.map
                    .field
                    .iter()
                    .flatten()
                    .map(|v| v.players.len())
                    .sum::<usize>(),
                1,
                "There is only one player on the map"
            );

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

        #[rstest]
        // See command tests
        // Doesn't see himself on the cell but resource
        #[case((1, Position {x: 1,y: 2,dir: North,}),
            // Other players
            vec![],
            // Resources
            vec![((1, 2), Nourriture)],
            // Expected answer in order (cell1, cell2, cell3 ..)
            vec!["nourriture", "", "", ""]
        )]
        // See other player on the cell
        #[case((1, Position {x: 1,y: 2,dir: North,}),
            // Other players
            vec![Position {x:1,y:2,dir:North}],
            // Resources
            vec![],
            // Expected answer in order (cell1, cell2, cell3 ..)
            vec!["player", "", "", ""]
        )]
        // Multiple same resources on the same cell
        #[case((1, Position {x: 1,y: 2,dir: North,}),
            // Other players
            vec![],
            // Resources
            vec![((1, 2), Nourriture), ((1, 2),Nourriture),
                ((0, 1), Stone(Deraumere)),((0, 1), Stone(Deraumere)),
                ((1, 1), Stone(Linemate)), ((1, 1), Stone(Linemate)),
                ((2, 1), Stone(Mendiane)), ((2, 1), Stone(Mendiane))
            ],
            // Expected answer in order (cell1, cell2, cell3 ..)
            vec!["nourriture nourriture", "deraumere deraumere", "linemate linemate", "mendiane mendiane"]
        )]
        // Can see via west border
        #[case((1, Position {x: 0,y: 1,dir: West,}),
            // Other players
            vec![Position {x: 2,y: 0,dir: West,}, Position {x: 2,y: 1,dir: West,}, Position {x: 2,y: 2,dir: West,}],
            // Resources
            vec![((0, 1), Nourriture), ((0, 1),Nourriture),
                ((2, 2), Stone(Mendiane)), ((2, 2), Stone(Thystame)),
                ((2, 1), Stone(Sibur)), ((2, 1), Nourriture),
                ((2, 0), Nourriture),((2, 0), Stone(Phiras)),
            ],
            // Expected answer in order (cell1, cell2, cell3 ..)
            vec!["nourriture nourriture",
                "player mendiane thystame",
                "player nourriture sibur",
                "player nourriture phiras"]
        )]
        // Can see via east border
        #[case((1, Position {x: 2,y: 1,dir: East,}),
            // Other players
            vec![Position {x: 0,y: 0,dir: West,}, Position {x: 0,y: 1,dir: West,}, Position {x: 0,y: 2,dir: West,}],
            // Resources
            vec![((2, 1), Stone(Mendiane)), ((2, 1),Stone(Deraumere)),
                ((0, 0), Stone(Mendiane)), ((0, 0), Stone(Thystame)),
                ((0, 1), Nourriture), ((0, 1), Nourriture), ((0, 1), Nourriture),
                ((0, 2), Nourriture),((0, 2), Stone(Sibur)), ((0, 2), Stone(Sibur)), ((0, 2), Stone(Sibur)),
            ],
            // Expected answer in order (cell1, cell2, cell3 ..)
            vec!["deraumere mendiane",
                "player mendiane thystame",
                "player nourriture nourriture nourriture",
                "player nourriture sibur sibur sibur"]
        )]
        // Can see via north border
        #[case((1, Position {x: 1,y: 0,dir: North,}),
            // Other players
            vec![Position {x: 0,y: 2,dir: West,}, Position {x: 1,y: 2,dir: West,}, Position {x: 2,y: 2,dir: West,}],
            // Resources
            vec![((1, 0), Stone(Mendiane)), ((1, 0),Stone(Deraumere)),
                ((0, 2), Stone(Mendiane)), ((0, 2), Stone(Thystame)),
                ((1, 2), Nourriture), ((1, 2), Nourriture), ((1, 2), Nourriture),
                ((2, 2), Nourriture),((2, 2), Stone(Sibur)), ((2, 2), Stone(Sibur)), ((2, 2), Stone(Sibur)),
            ],
            // Expected answer in order (cell1, cell2, cell3 ..)
            vec!["deraumere mendiane",
                "player mendiane thystame",
                "player nourriture nourriture nourriture",
                "player nourriture sibur sibur sibur"]
        )]
        // Can see via south border (not the american one)
        #[case((1, Position {x: 1,y: 2,dir: South,}),
            // Other players
            vec![Position {x: 0,y: 0,dir: West,}, Position {x: 1,y: 0,dir: West,}, Position {x: 2,y: 0,dir: West,}],
            // Resources
            vec![((1, 2), Stone(Mendiane)), ((1, 2),Stone(Deraumere)),
                ((2, 0), Nourriture),((2, 0), Stone(Sibur)), ((2, 0), Stone(Sibur)), ((2, 0), Stone(Sibur)),
                ((1, 0), Nourriture), ((1, 0), Nourriture), ((1, 0), Nourriture),
                ((0, 0), Stone(Mendiane)), ((0, 0), Stone(Thystame)),
            ],
            // Expected answer in order (cell1, cell2, cell3 ..)
            vec!["deraumere mendiane",
                "player nourriture sibur sibur sibur",
                "player nourriture nourriture nourriture",
                "player mendiane thystame",]
        )]
        // Stone, player and nourriture on the same cell
        #[case((1, Position {x: 1,y: 2,dir: North,}),
            // Other players
            vec![Position {x:1,y:1,dir:North}],
            // Resources
            vec![((1, 1), Nourriture), ((1, 1),Stone(Linemate))],
            // Expected answer in order (cell1, cell2, cell3 ..)
            vec!["", "", "player nourriture linemate", ""]
        )]
        // Can see recursively (level2)
        #[case((2, Position {x: 1,y: 0,dir: North,}),
            // Other players
            vec![Position {x: 0,y: 2,dir: West,}, Position {x: 1,y: 2,dir: West,}, Position {x: 2,y: 2,dir: West,}],
            // Resources
            vec![((1, 0), Stone(Mendiane)),
                ((0, 2), Stone(Sibur)),
                ((1, 2), Nourriture),
                ((2, 2), Stone(Thystame)),
            ],
            // Expected answer in order (cell1, cell2, cell3 ..)
            vec!["mendiane", "player sibur", "player nourriture", "player thystame", // level 1
                "", "", "", "", ""] //level 2
        )]
        // Can see recursively (level3)
        #[case((3, Position {x: 1,y: 0,dir: North,}),
            // Other players
            vec![Position {x: 0,y: 2,dir: West,}, Position {x: 1,y: 2,dir: West,}, Position {x: 2,y: 2,dir: West,}],
            // Resources
            vec![((1, 0), Stone(Mendiane)),
                ((0, 2), Stone(Sibur)),
                ((1, 2), Nourriture),
                ((2, 2), Stone(Thystame)),
            ],
            // Expected answer in order (cell1, cell2, cell3 ..)
            vec!["mendiane", "player sibur", "player nourriture", "player thystame", //level 1
                "", "", "", "", "", //level 2
                "mendiane", "", "", "mendiane", "", "", "mendiane"] //level 3
        )]
        // Can see recursively (level4)
        #[case((4, Position {x: 1,y: 0,dir: North,}),
            // Other players
            vec![Position {x: 0,y: 2,dir: West,}, Position {x: 1,y: 2,dir: West,}, Position {x: 2,y: 2,dir: West,}],
            // Resources
            vec![((1, 0), Stone(Mendiane)),
                ((0, 2), Stone(Sibur)),
                ((1, 2), Nourriture),
                ((2, 2), Stone(Thystame)),
            ],
            // Expected answer in order (cell1, cell2, cell3 ..)
            vec!["mendiane", "player sibur", "player nourriture", "player thystame", // level 1
                "", "", "", "", "", // level 2
                "mendiane", "", "", "mendiane", "", "", "mendiane", // level 3
                // level 4
                "player sibur", "player nourriture", "player thystame",
                "player sibur", "player nourriture", "player thystame",
                "player sibur", "player nourriture", "player thystame",]
        )]
        fn successfully_applies_see_command(
            #[case] player_under_test: (u8, Position),
            #[case] players: Vec<Position>,
            #[case] resource: Vec<((usize, usize), Resource)>,
            #[case] result: Vec<&str>,
        ) {
            // Given
            let result = result.iter().map(|s| s.to_string()).collect::<Vec<_>>();
            let all_players = vec![player_under_test.1]
                .iter()
                .chain(players.iter())
                .cloned()
                .collect::<Vec<_>>();

            let (players_ids, mut game) = game_engine_with(&all_players, Some(&resource));
            let player_under_test_id = players_ids[0];
            let mut execution_results_buffer = Vec::new();
            player_lvl_up(
                game.players.get_mut(&player_under_test_id).unwrap(),
                player_under_test.0,
            );
            let command = PlayerCmd::See;
            game.take_command(&player_under_test_id, command.clone())
                .unwrap();

            // When
            for _ in 0..command.delay() {
                game.tick(&mut execution_results_buffer)
            }

            // Then
            assert_eq!(execution_results_buffer.len(), 1);
            assert_eq!(
                execution_results_buffer[0],
                (player_under_test_id, ServerResponse::See(result))
            );
        }

        #[rstest]
        // Take test for stones
        // Successfully takes a stone from sell
        #[case(
            vec![Stone(Linemate)], // Cell initial content
            vec![PlayerCmd::Take(Linemate.to_string())], // Command in order to execute
            vec![ServerResponse::Ok], // expected response
           //D  L  M  P  S  T | Player final inventory
            [0, 1, 0, 0, 0, 0],
           //D  L  M  P  S  T | Final cell content
            [0, 0, 0, 0, 0, 0],
        )]
        // Can't take a stone if it is not on the cell
        #[case(
            vec![],  // Cell initial content
            vec![PlayerCmd::Take(Thystame.to_string())], // Command in order to execute
            vec![ServerResponse::Ko], // expected response
           //D  L  M  P  S  T | Player final inventory
            [0, 0, 0, 0, 0, 0],
           //D  L  M  P  S  T | Final cell content
            [0, 0, 0, 0, 0, 0],
        )]
        // Can't take a nonexistent stone
        #[case(
            vec![Stone(Sibur)], // Cell initial content
            vec![PlayerCmd::Take("💎SAPPHIRE💎".to_string())], // Command in order to execute
            vec![ServerResponse::Ko], // expected response
           //D  L  M  P  S  T | Player final inventory
            [0, 0, 0, 0, 0, 0],
           //D  L  M  P  S  T | Final cell content
            [0, 0, 0, 0, 1, 0],
        )]
        // Can have multiple stones
        #[case(
            vec![Stone(Sibur), Stone(Sibur)], // Cell initial content
            vec![PlayerCmd::Take(Sibur.to_string()), PlayerCmd::Take(Sibur.to_string())], // Command in order to execute
            vec![ServerResponse::Ok, ServerResponse::Ok], // expected response
           //D  L  M  P  S  T | Player final inventory
            [0, 0, 0, 0, 2, 0],
           //D  L  M  P  S  T | Final cell content
            [0, 0, 0, 0, 0, 0],
        )]
        // Tries to take each stone, when it is on the sell and not
        #[case(vec![
            // Cell initial content
            Stone(Thystame),
            Stone(Sibur),
            Stone(Deraumere),
            Stone(Linemate),
            Stone(Mendiane),
            Stone(Phiras),
        ], vec![
            // Command in order to execute
            PlayerCmd::Take(Thystame.to_string()), PlayerCmd::Take(Thystame.to_string()),
            PlayerCmd::Take(Deraumere.to_string()), PlayerCmd::Take(Deraumere.to_string()),
            PlayerCmd::Take(Mendiane.to_string()), PlayerCmd::Take(Mendiane.to_string()),
            PlayerCmd::Take(Phiras.to_string()), PlayerCmd::Take(Phiras.to_string()),
            PlayerCmd::Take(Sibur.to_string()), PlayerCmd::Take(Sibur.to_string()),
            PlayerCmd::Take(Linemate.to_string()), PlayerCmd::Take(Linemate.to_string()),
        ], vec![
            // expected response
            ServerResponse::Ok, ServerResponse::Ko,
            ServerResponse::Ok, ServerResponse::Ko,
            ServerResponse::Ok, ServerResponse::Ko,
            ServerResponse::Ok, ServerResponse::Ko,
            ServerResponse::Ok, ServerResponse::Ko,
            ServerResponse::Ok, ServerResponse::Ko,
        ],
           //D  L  M  P  S  T | Player final inventory
            [1, 1, 1, 1, 1, 1],
           //D  L  M  P  S  T | Final cell content
            [0, 0, 0, 0, 0, 0],
        )]
        // Put tests for stones

        // Can't put a stone if it is not in the inventory
        #[case(
            vec![], // Cell initial content
            vec![PlayerCmd::Put(Linemate.to_string())], // Command in order to execute
            vec![ServerResponse::Ko], // expected response
           //D  L  M  P  S  T | Player final inventory
            [0, 0, 0, 0, 0, 0],
           //D  L  M  P  S  T | Final cell content
            [0, 0, 0, 0, 0, 0],
        )]
        // Successfully puts the thystame from inventory on cell but can't put linemate
        // because it is not in the inventory
        #[case(
            vec![Stone(Thystame)], // Cell initial content
            vec![PlayerCmd::Take(Thystame.to_string()), PlayerCmd::Put(Linemate.to_string())], // Command in order to execute
            vec![ServerResponse::Ok, ServerResponse::Ko], // expected response
           //D  L  M  P  S  T | Player final inventory
            [0, 0, 0, 0, 0, 1],
           //D  L  M  P  S  T | Final cell content
            [0, 0, 0, 0, 0, 0],
        )]
        // Successfully takes a stone from a cell and then successfully puts it back
        #[case(
            vec![Stone(Thystame)], // Cell initial content
            vec![PlayerCmd::Take(Thystame.to_string()), PlayerCmd::Put(Thystame.to_string())], // Command in order to execute
            vec![ServerResponse::Ok, ServerResponse::Ok], // expected response
           //D  L  M  P  S  T | Player final inventory
            [0, 0, 0, 0, 0, 0],
           //D  L  M  P  S  T | Final cell content
            [0, 0, 0, 0, 0, 1],
        )]
        // Successfully takes a stone, and then tries to put a nonexistent stone on the cell
        #[case(
            vec![Stone(Thystame)], // Cell initial content
            vec![PlayerCmd::Take(Thystame.to_string()), PlayerCmd::Put("💎SAPPHIRE💎".to_string())], // Command in order to execute
            vec![ServerResponse::Ok, ServerResponse::Ko], // expected response
           //D  L  M  P  S  T | Player final inventory
            [0, 0, 0, 0, 0, 1],
           //D  L  M  P  S  T | Final cell content
            [0, 0, 0, 0, 0, 0],
        )]
        // Successfully takes and then successfully puts every existent type of stone
        #[case(vec![
            // Cell initial content
            Stone(Thystame),
            Stone(Sibur),
            Stone(Deraumere),
            Stone(Linemate),
            Stone(Mendiane),
            Stone(Phiras),
        ], vec![
            // Command in order to execute
        PlayerCmd::Take(Thystame.to_string()), PlayerCmd::Put(Thystame.to_string()),
            PlayerCmd::Take(Deraumere.to_string()), PlayerCmd::Put(Deraumere.to_string()),
            PlayerCmd::Take(Mendiane.to_string()), PlayerCmd::Put(Mendiane.to_string()),
            PlayerCmd::Take(Phiras.to_string()), PlayerCmd::Put(Phiras.to_string()),
            PlayerCmd::Take(Sibur.to_string()), PlayerCmd::Put(Sibur.to_string()),
            PlayerCmd::Take(Linemate.to_string()), PlayerCmd::Put(Linemate.to_string()),
        ], vec![
            // expected response
            ServerResponse::Ok, ServerResponse::Ok,
            ServerResponse::Ok, ServerResponse::Ok,
            ServerResponse::Ok, ServerResponse::Ok,
            ServerResponse::Ok, ServerResponse::Ok,
            ServerResponse::Ok, ServerResponse::Ok,
            ServerResponse::Ok, ServerResponse::Ok,
        ],
           //D  L  M  P  S  T | Player final inventory
            [0, 0, 0, 0, 0, 0],
           //D  L  M  P  S  T | Final cell content
            [1, 1, 1, 1, 1, 1],
        )]
        /* Tries to put on the cell every type of stone, but the inventory is empty and
           there already some stones on the cell. It ensures we don't modify the cell
           during with a failed put
        */
        #[case(vec![
            // Cell initial content
            Stone(Thystame),
            Stone(Sibur),
            Stone(Deraumere),
            Stone(Linemate),
            Stone(Mendiane),
            Stone(Phiras),
        ], vec![
            // Command in order to execute
            PlayerCmd::Put(Thystame.to_string()),
            PlayerCmd::Put(Deraumere.to_string()),
            PlayerCmd::Put(Mendiane.to_string()),
            PlayerCmd::Put(Phiras.to_string()),
            PlayerCmd::Put(Sibur.to_string()),
            PlayerCmd::Put(Linemate.to_string()),
        ], vec![
            // expected response
            ServerResponse::Ko,
            ServerResponse::Ko,
            ServerResponse::Ko,
            ServerResponse::Ko,
            ServerResponse::Ko,
            ServerResponse::Ko,
        ],
           //D  L  M  P  S  T | Player final inventory
            [0, 0, 0, 0, 0, 0],
           //D  L  M  P  S  T | Final cell content
            [1, 1, 1, 1, 1, 1],
        )]
        fn applies_take_and_put_commands_for_stones(
            #[case] resource: Vec<Resource>,
            #[case] commands: Vec<PlayerCmd>,
            #[case] result: Vec<ServerResponse>,
            #[case] final_inventory: StoneSet,
            #[case] final_cell_content: StoneSet,
        ) {
            // Given
            let position = Position {
                x: 1,
                y: 1,
                dir: West,
            };
            let (players_ids, mut game) = game_engine_with(
                &vec![position],
                Some(
                    &resource
                        .iter()
                        .map(|v| ((position.x, position.y), v.clone()))
                        .collect::<Vec<_>>(),
                ),
            );
            let player_under_test_id = players_ids[0];
            let result = result
                .iter()
                .map(|response| (player_under_test_id, response))
                .collect::<Vec<_>>();
            let mut execution_results_buffer = Vec::new();

            //When
            for command in &commands {
                game.take_command(&player_under_test_id, command.clone())
                    .unwrap();
                for _ in 0..command.delay() {
                    game.tick(&mut execution_results_buffer);
                }
            }

            //Then
            let player_under_test = game.players.get(&players_ids[0]).unwrap();
            assert!(execution_results_buffer
                .iter()
                .zip(result)
                .all(|(a, b)| a.0 == b.0 && a.1 == *b.1));
            assert_eq!(
                game.map.field[position.y][position.x].stones,
                final_cell_content
            );
            assert_eq!(game.map.field[position.y][position.x].nourriture, 0);
            assert_eq!(player_under_test.inventory(), &final_inventory);
            assert_eq!(resources_sum_on_other_cell(&player_under_test_id, &game), 0);
        }

        #[rstest]
        #[case(vec![], vec![PlayerCmd::Put(Nourriture.to_string())], 1, 1, 0, vec![ServerResponse::Ko])]
        #[case(vec![], vec![PlayerCmd::Put(Nourriture.to_string())], LIFE_TICKS + 1, 1, 1, vec![ServerResponse::Ok])]
        #[case(vec![], vec![PlayerCmd::Take(Nourriture.to_string())], 1, 1, 0, vec![ServerResponse::Ko])]
        #[case(vec![Nourriture], vec![PlayerCmd::Take(Nourriture.to_string())], 1, 1 + LIFE_TICKS, 0, vec![ServerResponse::Ok])]
        fn applies_take_and_put_commands_for_nourriture(
            #[case] resource: Vec<Resource>,
            #[case] commands: Vec<PlayerCmd>,
            #[case] initial_hp: u64,
            #[case] final_hp: u64,
            #[case] final_cell_nourriture_count: usize,
            #[case] responses: Vec<ServerResponse>,
        ) {
            // Given
            let position = Position {
                x: 1,
                y: 1,
                dir: West,
            };
            let (players_ids, mut game) = game_engine_with(
                &vec![position],
                Some(
                    &resource
                        .iter()
                        .map(|v| ((position.x, position.y), v.clone()))
                        .collect::<Vec<_>>(),
                ),
            );
            let player_under_test_id = players_ids[0];
            let responses = responses
                .into_iter()
                .map(|v| (player_under_test_id, v))
                .collect::<Vec<_>>();
            let mut execution_results_buffer = Vec::new();
            let all_cmd_delay = commands.iter().map(|command| command.delay()).sum::<u64>();
            let initial_hp = initial_hp + all_cmd_delay;
            player_set_hp(
                game.players.get_mut(&player_under_test_id).unwrap(),
                initial_hp,
            );

            //When
            for command in &commands {
                game.take_command(&player_under_test_id, command.clone())
                    .unwrap();
                for _ in 0..command.delay() {
                    game.tick(&mut execution_results_buffer);
                }
            }

            //Then
            let player_under_test = game.players.get(&player_under_test_id).unwrap();
            assert_eq!(
                game.map.field[position.y][position.x].nourriture,
                final_cell_nourriture_count
            );
            assert_eq!(
                game.map.field[position.y][position.x]
                    .stones
                    .iter()
                    .map(|v| *v as u64)
                    .sum::<u64>(),
                0
            );
            assert_eq!(resources_sum_on_other_cell(&player_under_test_id, &game), 0);
            assert_eq!(*player_under_test.remaining_life(), final_hp);
            assert_eq!(execution_results_buffer.len(), 1);
            assert_eq!(execution_results_buffer, responses);
        }

        #[rstest]
        // (player inventory, player hp)
        #[case([0, 0, 0, 0, 5, 0], 1)]
        #[case([1, 0, 0, 4, 0, 0], 255)]
        #[case([0, 1, 1, 0, 8, 0], 8123)]
        #[case([9, 2, 4, 0, 0, 0], 34)]
        #[case([0, 0, 0, 0, 0, 0], 8841241)]
        #[case([42, 42, 42, 42, 22, 42], 29)]
        #[case([2, 0, 4912, 0, 8, 0], 42)]
        #[case([1, 1, 5, 5, 5, 5], 342)]
        fn applies_inventory_command(#[case] player_inventory: StoneSet, #[case] player_hp: u64) {
            // Given
            let (player_id, mut game) = one_player_game_engine();
            let command = PlayerCmd::Inventory;
            let mut execution_results_buffer = Vec::new();
            game.take_command(&player_id, command.clone()).unwrap();
            let player = game.players.get_mut(&player_id).unwrap();

            player_set_hp(player, player_hp + command.delay());
            for (i, count) in player_inventory.iter().enumerate() {
                for _ in 0..*count {
                    player.add_to_inventory(Resource::try_from(i).unwrap());
                }
            }

            let expected_result = vec![
                format!("nourriture {}", player_hp),
                format!("deraumere {}", player_inventory[0]),
                format!("linemate {}", player_inventory[1]),
                format!("mendiane {}", player_inventory[2]),
                format!("phiras {}", player_inventory[3]),
                format!("sibur {}", player_inventory[4]),
                format!("thystame {}", player_inventory[5]),
            ];

            //When
            for _ in 0..command.delay() {
                game.tick(&mut execution_results_buffer);
            }

            // Then
            assert_eq!(execution_results_buffer.len(), 1);
            assert_eq!(
                execution_results_buffer[0],
                (player_id, ServerResponse::Inventory(expected_result))
            );
        }
    }
}
