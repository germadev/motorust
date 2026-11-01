//! The engine's checks run on the game's content: each a test of an engine
//! module's, written to take the circuits (or the bikes) it checks, which
//! the engine runs on its own demo content and the game here on its own
//! circuits and bikes (step 4c of the engine's split, `ROADMAP.md`). Those
//! of what the engine's demo has none of (loops, splits, portals, a mine,
//! a dam...) only the game runs.

use super::{CIRCUITS, ROSTER};

mod bike_ai {
    use super::*;

    #[test]
    fn rival_laps_the_track_without_leaving_it() {
        crate::bike::ai::tests::rival_laps_the_track_without_leaving_it_on(&CIRCUITS);
    }

    #[test]
    fn against_the_clock_a_good_ride_earns_a_little_time_at_each_checkpoint() {
        crate::bike::ai::tests::against_the_clock_a_good_ride_earns_a_little_time_at_each_checkpoint_on(&CIRCUITS);
    }

    #[test]
    fn rivals_that_take_the_branches_ride_round_the_loops_and_back() {
        crate::bike::ai::tests::rivals_that_take_the_branches_ride_round_the_loops_and_back_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn rivals_ride_the_forks_on_their_side_and_back() {
        crate::bike::ai::tests::rivals_ride_the_forks_on_their_side_and_back_on(&CIRCUITS);
    }

    #[test]
    fn no_way_through_a_split_is_clearly_the_quickest() {
        crate::bike::ai::tests::no_way_through_a_split_is_clearly_the_quickest_on(&CIRCUITS);
    }

    #[test]
    fn round_a_loop_is_about_as_quick_as_the_lap_beside_it() {
        crate::bike::ai::tests::round_a_loop_is_about_as_quick_as_the_lap_beside_it_on(&CIRCUITS);
    }

    #[test]
    fn out_of_a_loop_the_road_back_keeps_the_wheels_on_the_ground() {
        crate::bike::ai::tests::out_of_a_loop_the_road_back_keeps_the_wheels_on_the_ground_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn round_a_loop_a_rival_gets_to_its_line_clear_of_the_walls() {
        crate::bike::ai::tests::round_a_loop_a_rival_gets_to_its_line_clear_of_the_walls_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn a_pack_of_rivals_races_without_wrecking_each_other() {
        crate::bike::ai::tests::a_pack_of_rivals_races_without_wrecking_each_other_on(&CIRCUITS);
    }

    #[test]
    fn a_very_hard_pack_races_without_wrecking_each_other() {
        crate::bike::ai::tests::a_very_hard_pack_races_without_wrecking_each_other_on(&CIRCUITS);
    }

    #[test]
    fn on_very_hard_the_rivals_are_quicker_than_on_hard_and_keep_to_the_road() {
        crate::bike::ai::tests::on_very_hard_the_rivals_are_quicker_than_on_hard_and_keep_to_the_road_on(&CIRCUITS);
    }

    #[test]
    fn the_grid_gets_away_without_a_touch() {
        crate::bike::ai::tests::the_grid_gets_away_without_a_touch_on(&CIRCUITS);
    }

    #[test]
    fn jumps_on_the_track_are_short() {
        crate::bike::ai::tests::jumps_on_the_track_are_short_on(&CIRCUITS);
    }

    #[test]
    fn turbo_bay_s_seafront_runs_into_its_first_bend_with_no_crest() {
        crate::bike::ai::tests::turbo_bay_s_seafront_runs_into_its_first_bend_with_no_crest_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn crests_only_throw_the_bikes_into_the_air_on_the_straights() {
        crate::bike::ai::tests::crests_only_throw_the_bikes_into_the_air_on_the_straights_on(
            &CIRCUITS,
        );
    }
}

mod world_cavern {
    use super::*;

    #[test]
    fn moria_s_great_hall_is_far_wider_and_higher_than_the_road_past_its_pillars() {
        crate::world::cavern::tests::moria_s_great_hall_is_far_wider_and_higher_than_the_road_past_its_pillars_on(&CIRCUITS);
    }

    #[test]
    fn the_halls_are_high_and_the_galleries_lower() {
        crate::world::cavern::tests::the_halls_are_high_and_the_galleries_lower_on(&CIRCUITS);
    }

    #[test]
    fn the_walls_stand_where_the_bikes_hit_them_with_the_roof_above_a_rider() {
        crate::world::cavern::tests::the_walls_stand_where_the_bikes_hit_them_with_the_roof_above_a_rider_on(&CIRCUITS);
    }

    #[test]
    fn rock_stands_between_the_roads_where_they_part() {
        crate::world::cavern::tests::rock_stands_between_the_roads_where_they_part_on(&CIRCUITS);
    }

    #[test]
    fn the_rock_over_the_excavation_carries_the_road_over_the_top() {
        crate::world::cavern::tests::the_rock_over_the_excavation_carries_the_road_over_the_top_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn the_excavation_is_closed_and_lit() {
        crate::world::cavern::tests::the_excavation_is_closed_and_lit_on(&CIRCUITS);
    }
}

mod bike_crash {
    use super::*;

    #[test]
    fn a_bike_that_falls_downhill_does_not_float_off_the_road() {
        crate::bike::crash::tests::a_bike_that_falls_downhill_does_not_float_off_the_road_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn a_fallen_bike_slides_further_down_a_hill_than_up_it() {
        crate::bike::crash::tests::a_fallen_bike_slides_further_down_a_hill_than_up_it_on(
            &CIRCUITS,
        );
    }
}

mod world_dam {
    use super::*;

    #[test]
    fn turbo_bay_comes_down_onto_its_dam_straight_at_it() {
        crate::world::dam::tests::turbo_bay_comes_down_onto_its_dam_straight_at_it_on(&CIRCUITS);
    }

    #[test]
    fn turbo_bay_s_lake_shows_coming_down_to_the_dam_and_over_its_parapet() {
        crate::world::dam::tests::turbo_bay_s_lake_shows_coming_down_to_the_dam_and_over_its_parapet_on(&CIRCUITS);
    }

    #[test]
    fn turbo_bay_s_dam_holds_its_lake_a_few_metres_under_the_crest() {
        crate::world::dam::tests::turbo_bay_s_dam_holds_its_lake_a_few_metres_under_the_crest_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn turbo_bay_s_river_runs_from_the_foot_of_the_dam_down_the_gorge_to_the_sea() {
        crate::world::dam::tests::turbo_bay_s_river_runs_from_the_foot_of_the_dam_down_the_gorge_to_the_sea_on(&CIRCUITS);
    }

    #[test]
    fn turbo_bay_s_dam_face_drops_to_the_river_in_sight_of_the_road_up_the_gorge() {
        crate::world::dam::tests::turbo_bay_s_dam_face_drops_to_the_river_in_sight_of_the_road_up_the_gorge_on(&CIRCUITS);
    }

    #[test]
    fn every_dam_s_crest_is_a_bridge_of_its_own() {
        crate::world::dam::tests::every_dam_s_crest_is_a_bridge_of_its_own_on(&CIRCUITS);
    }

    #[test]
    fn a_lake_s_water_is_a_fan_that_never_folds_over() {
        crate::world::dam::tests::a_lake_s_water_is_a_fan_that_never_folds_over_on(&CIRCUITS);
    }

    #[test]
    fn a_dam_s_crest_bows_towards_its_lake() {
        crate::world::dam::tests::a_dam_s_crest_bows_towards_its_lake_on(&CIRCUITS);
    }

    #[test]
    fn turbo_bay_s_lake_mirrors_the_hills_round_it_and_the_dam() {
        crate::world::dam::tests::turbo_bay_s_lake_mirrors_the_hills_round_it_and_the_dam_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn the_sun_shines_on_turbo_bay_s_dam_face_all_across_it() {
        crate::world::dam::tests::the_sun_shines_on_turbo_bay_s_dam_face_all_across_it_on(
            &CIRCUITS,
        );
    }
}

mod bike_model_designs {
    use super::*;

    #[test]
    fn designs_index_in_order() {
        crate::bike::model::designs::tests::designs_index_in_order_on(&ROSTER);
    }
}

mod world_flowers {
    use super::*;

    #[test]
    fn most_of_the_grass_is_in_flower_but_the_shoulders() {
        crate::world::flowers::tests::most_of_the_grass_is_in_flower_but_the_shoulders_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn clumps_stand_near_the_roads_in_their_fields_and_not_too_many() {
        crate::world::flowers::tests::clumps_stand_near_the_roads_in_their_fields_and_not_too_many_on(&CIRCUITS);
    }

    #[test]
    fn clumps_keep_clear_of_what_stands_on_the_ground() {
        crate::world::flowers::tests::clumps_keep_clear_of_what_stands_on_the_ground_on(&CIRCUITS);
    }

    #[test]
    fn each_square_of_clumps_is_one_mesh_of_three_cards_a_clump() {
        crate::world::flowers::tests::each_square_of_clumps_is_one_mesh_of_three_cards_a_clump_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn fields_thin_out_and_mingle_at_their_borders() {
        crate::world::flowers::tests::fields_thin_out_and_mingle_at_their_borders_on(&CIRCUITS);
    }
}

mod bike_model_imported {
    use super::*;

    #[test]
    fn bikes_have_their_parts_and_wheels_on_the_ground() {
        crate::bike::model::imported::tests::bikes_have_their_parts_and_wheels_on_the_ground_on(
            &ROSTER,
        );
    }

    #[test]
    fn exhausts_open_at_the_back_of_the_silencers() {
        crate::bike::model::imported::tests::exhausts_open_at_the_back_of_the_silencers_on(&ROSTER);
    }

    #[test]
    fn pilots_have_a_rigged_body() {
        crate::bike::model::imported::tests::pilots_have_a_rigged_body_on(&ROSTER);
    }
}

mod sound_music {
    use super::*;

    #[test]
    fn every_song_is_in_the_pack() {
        crate::sound::music::tests::every_song_is_in_the_pack_on(&CIRCUITS);
    }

    #[test]
    fn every_song_is_turned_to_its_loudness() {
        crate::sound::music::tests::every_song_is_turned_to_its_loudness_on(&CIRCUITS);
    }

    #[test]
    fn songs_last_as_long_as_they_say_and_loop_back_to_the_start() {
        crate::sound::music::tests::songs_last_as_long_as_they_say_and_loop_back_to_the_start_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn the_music_stays_well_under_the_player_s_engine() {
        crate::sound::music::tests::the_music_stays_well_under_the_player_s_engine_on(&CIRCUITS);
    }

    #[test]
    fn the_menus_have_their_song_the_race_the_track_s_and_the_bikes_none() {
        crate::sound::music::tests::the_menus_have_their_song_the_race_the_track_s_and_the_bikes_none_on(&CIRCUITS);
    }

    #[test]
    fn a_song_of_a_list_played_through_gives_way_to_the_next() {
        crate::sound::music::tests::a_song_of_a_list_played_through_gives_way_to_the_next_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn with_no_songs_nothing_plays_and_free_ride_keeps_to_what_has_some() {
        crate::sound::music::tests::with_no_songs_nothing_plays_and_free_ride_keeps_to_what_has_some_on(&CIRCUITS);
    }

    #[test]
    fn free_ride_plays_the_track_s_song_then_every_track_s_then_chill() {
        crate::sound::music::tests::free_ride_plays_the_track_s_song_then_every_track_s_then_chill_on(&CIRCUITS);
    }
}

mod bike_model {
    use super::*;

    #[test]
    fn every_bike_reaches_no_further_than_its_nose_and_tail() {
        crate::bike::model::tests::every_bike_reaches_no_further_than_its_nose_and_tail_on(&ROSTER);
    }

    #[test]
    fn plates_face_outwards() {
        crate::bike::model::tests::plates_face_outwards_on(&ROSTER);
    }
}

mod bike_model_pilot {
    use super::*;

    #[test]
    fn hands_reach_the_grips_and_feet_the_pegs_on_every_bike() {
        crate::bike::model::pilot::tests::hands_reach_the_grips_and_feet_the_pegs_on_every_bike_on(
            &ROSTER,
        );
    }

    #[test]
    fn right_foot_reaches_the_ground_when_stopped() {
        crate::bike::model::pilot::tests::right_foot_reaches_the_ground_when_stopped_on(&ROSTER);
    }

    #[test]
    fn bones_keep_their_lengths() {
        crate::bike::model::pilot::tests::bones_keep_their_lengths_on(&ROSTER);
    }

    #[test]
    fn tucked_pilot_lies_over_the_tank_and_looks_ahead() {
        crate::bike::model::pilot::tests::tucked_pilot_lies_over_the_tank_and_looks_ahead_on(
            &ROSTER,
        );
    }

    #[test]
    fn head_turns_towards_the_corner_without_tilting() {
        crate::bike::model::pilot::tests::head_turns_towards_the_corner_without_tilting_on(&ROSTER);
    }

    #[test]
    fn on_the_straight_the_pilot_tucks_in_behind_the_screen() {
        crate::bike::model::pilot::tests::on_the_straight_the_pilot_tucks_in_behind_the_screen_on(
            &ROSTER,
        );
    }

    #[test]
    fn inner_knee_goes_out_in_a_corner() {
        crate::bike::model::pilot::tests::inner_knee_goes_out_in_a_corner_on(&ROSTER);
    }
}

mod world_realm {
    use super::*;

    #[test]
    fn one_goes_from_one_realm_to_the_other_only_through_a_portal() {
        crate::world::realm::tests::one_goes_from_one_realm_to_the_other_only_through_a_portal_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn only_moon_base_is_split() {
        crate::world::realm::tests::only_moon_base_is_split_on(&CIRCUITS);
    }
}

mod bench {
    use super::*;

    #[test]
    fn every_track_is_ridden_under_each_setting_with_one_change_at_a_time() {
        crate::bench::tests::every_track_is_ridden_under_each_setting_with_one_change_at_a_time_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn the_ground_is_left_out_of_the_shadows_only_in_its_turn() {
        crate::bench::tests::the_ground_is_left_out_of_the_shadows_only_in_its_turn_on(&CIRCUITS);
    }

    #[test]
    fn two_samples_are_left_out_where_the_gpu_has_not_got_them() {
        crate::bench::tests::two_samples_are_left_out_where_the_gpu_has_not_got_them_on(&CIRCUITS);
    }

    #[test]
    fn the_tracks_asked_for_or_all() {
        crate::bench::tests::the_tracks_asked_for_or_all_on(&CIRCUITS);
    }

    #[test]
    fn the_results_read_as_a_table() {
        crate::bench::tests::the_results_read_as_a_table_on(&CIRCUITS);
    }

    #[test]
    fn a_run_takes_about_as_long_as_its_laps() {
        crate::bench::tests::a_run_takes_about_as_long_as_its_laps_on(&CIRCUITS);
    }
}

mod bike {
    use super::*;

    #[test]
    fn the_roster_reads_a_bike_for_each_place() {
        crate::bike::tests::the_roster_reads_a_bike_for_each_place_on(&ROSTER);
    }

    #[test]
    fn the_chosen_bike_takes_the_player_slot_and_the_rest_the_rivals() {
        crate::bike::tests::the_chosen_bike_takes_the_player_slot_and_the_rest_the_rivals_on(
            &ROSTER,
        );
    }

    #[test]
    fn the_toy_store_s_helix_is_taken_flat_out() {
        crate::bike::tests::the_toy_store_s_helix_is_taken_flat_out_on(&CIRCUITS);
    }

    #[test]
    fn flat_out_steering_only_laps_without_touching_the_walls() {
        crate::bike::tests::flat_out_steering_only_laps_without_touching_the_walls_on(&CIRCUITS);
    }
}

mod bike_wheelie_halo {
    use super::*;

    #[test]
    fn only_loop_island_leaves_a_blue_halo() {
        crate::bike::wheelie_halo::tests::only_loop_island_leaves_a_blue_halo_on(&CIRCUITS);
    }
}

mod circuits_data {
    use super::*;

    #[test]
    fn every_circuit_file_reads() {
        crate::circuits::data::tests::every_circuit_file_reads_on(&CIRCUITS);
    }
}

mod circuits {
    use super::*;

    #[test]
    fn circuits_have_checkpoints() {
        crate::circuits::tests::circuits_have_checkpoints_on(&CIRCUITS);
    }
}

mod menu {
    use super::*;

    #[test]
    fn split_screen_select_initializes_and_updates_without_conflict() {
        crate::menu::tests::split_screen_select_initializes_and_updates_without_conflict_on(
            &CIRCUITS, &ROSTER,
        );
    }
}

mod modes {
    use super::*;

    #[test]
    fn every_mode_has_a_name_and_a_description_in_each_language() {
        crate::modes::tests::every_mode_has_a_name_and_a_description_in_each_language_on(&CIRCUITS);
    }
}

mod race_hud {
    use super::*;

    #[test]
    fn the_top_left_clears_once_the_race_is_over_for_the_players() {
        crate::race::hud::tests::the_top_left_clears_once_the_race_is_over_for_the_players_on(
            &ROSTER,
        );
    }

    #[test]
    fn those_in_are_listed_by_when_they_crossed_the_line() {
        crate::race::hud::tests::those_in_are_listed_by_when_they_crossed_the_line_on(&ROSTER);
    }

    #[test]
    fn a_lap_record_is_told_once_just_after_the_lap_that_beat_it() {
        crate::race::hud::tests::a_lap_record_is_told_once_just_after_the_lap_that_beat_it_on(
            &ROSTER,
        );
    }

    #[test]
    fn back_on_the_grid_after_a_false_start_it_says_how_many() {
        crate::race::hud::tests::back_on_the_grid_after_a_false_start_it_says_how_many_on(&ROSTER);
    }
}

mod race {
    use super::*;

    #[test]
    fn every_track_has_its_checkpoints_evenly_spread() {
        crate::race::tests::every_track_has_its_checkpoints_evenly_spread_on(&CIRCUITS);
    }

    #[test]
    fn a_championship_adds_up_points_race_after_race() {
        crate::race::tests::a_championship_adds_up_points_race_after_race_on(&CIRCUITS);
    }

    #[test]
    fn a_bike_on_a_road_off_the_lap_is_worth_watching() {
        crate::race::tests::a_bike_on_a_road_off_the_lap_is_worth_watching_on(&CIRCUITS);
    }

    #[test]
    fn each_mode_has_its_laps() {
        crate::race::tests::each_mode_has_its_laps_on(&ROSTER);
    }
}

mod race_results {
    use super::*;

    #[test]
    fn a_player_disqualified_says_so_in_the_results() {
        crate::race::results::tests::a_player_disqualified_says_so_in_the_results_on(&ROSTER);
    }

    #[test]
    fn after_the_results_comes_what_the_mode_says() {
        crate::race::results::tests::after_the_results_comes_what_the_mode_says_on(&CIRCUITS);
    }
}

mod track {
    use super::*;

    #[test]
    fn walls_of_distant_stretches_do_not_meet() {
        crate::track::tests::walls_of_distant_stretches_do_not_meet_on(&CIRCUITS);
    }

    #[test]
    fn walls_close_in_on_a_narrow_bridge_and_open_out_after_it() {
        crate::track::tests::walls_close_in_on_a_narrow_bridge_and_open_out_after_it_on(&CIRCUITS);
    }

    #[test]
    fn the_banking_puts_no_bumps_off_the_middle_of_the_road() {
        crate::track::tests::the_banking_puts_no_bumps_off_the_middle_of_the_road_on(&CIRCUITS);
    }

    #[test]
    fn banked_spans_bank_more() {
        crate::track::tests::banked_spans_bank_more_on(&CIRCUITS);
    }

    #[test]
    fn the_grid_finds_what_going_through_every_segment_finds() {
        crate::track::tests::the_grid_finds_what_going_through_every_segment_finds_on(&CIRCUITS);
    }

    #[test]
    fn the_grid_finds_the_points_of_segments_corners_as_the_search_did() {
        crate::track::tests::the_grid_finds_the_points_of_segments_corners_as_the_search_did_on(
            &CIRCUITS,
        );
    }

    #[ignore = "a measurement, not a check"]
    #[test]
    fn time_to_find_a_bike_on_each_track() {
        crate::track::tests::time_to_find_a_bike_on_each_track_on(&CIRCUITS);
    }

    #[test]
    fn curves_are_drivable() {
        crate::track::tests::curves_are_drivable_on(&CIRCUITS);
    }

    #[test]
    fn start_faces_down_the_main_straight() {
        crate::track::tests::start_faces_down_the_main_straight_on(&CIRCUITS);
    }

    #[test]
    fn a_boosted_loop_has_a_booster_on_its_lead_in_and_one_beside_it_on_the_lap() {
        crate::track::tests::a_boosted_loop_has_a_booster_on_its_lead_in_and_one_beside_it_on_the_lap_on(&CIRCUITS);
    }

    #[test]
    fn slopes_and_banking_stay_rideable() {
        crate::track::tests::slopes_and_banking_stay_rideable_on(&CIRCUITS);
    }

    #[test]
    fn raised_banking_keeps_the_inside_where_it_would_lie_level() {
        crate::track::tests::raised_banking_keeps_the_inside_where_it_would_lie_level_on(&CIRCUITS);
    }

    #[test]
    fn a_branch_leaves_the_lap_and_joins_it_again_along_it() {
        crate::track::tests::a_branch_leaves_the_lap_and_joins_it_again_along_it_on(&CIRCUITS);
    }

    #[test]
    fn branches_are_rideable_and_clear_of_the_lap() {
        crate::track::tests::branches_are_rideable_and_clear_of_the_lap_on(&CIRCUITS);
    }

    #[test]
    fn branches_climb_and_drop_smoothly_on_their_own() {
        crate::track::tests::branches_climb_and_drop_smoothly_on_their_own_on(&CIRCUITS);
    }

    #[test]
    fn bikes_on_a_branch_find_its_road() {
        crate::track::tests::bikes_on_a_branch_find_its_road_on(&CIRCUITS);
    }

    #[test]
    fn a_fork_leaves_the_lap_and_joins_it_again_past_rock() {
        crate::track::tests::a_fork_leaves_the_lap_and_joins_it_again_past_rock_on(&CIRCUITS);
    }
}

mod track_racing_line {
    use super::*;

    #[test]
    fn on_every_track_it_stays_on_the_asphalt_and_takes_the_sharpest_bend_inside() {
        crate::track::racing_line::tests::on_every_track_it_stays_on_the_asphalt_and_takes_the_sharpest_bend_inside_on(&CIRCUITS);
    }
}

mod welcome {
    use super::*;

    #[test]
    fn the_camera_flies_under_the_rock_through_every_tunnel() {
        crate::welcome::tests::the_camera_flies_under_the_rock_through_every_tunnel_on(&CIRCUITS);
    }

    #[test]
    fn the_camera_flies_under_the_beam_of_every_arch_over_the_road() {
        crate::welcome::tests::the_camera_flies_under_the_beam_of_every_arch_over_the_road_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn the_flight_over_a_track_sets_off_just_before_its_line() {
        crate::welcome::tests::the_flight_over_a_track_sets_off_just_before_its_line_on(&CIRCUITS);
    }
}

mod world_landmarks {
    use super::*;

    #[test]
    fn landmarks_of_every_circuit_stand_clear_of_the_road() {
        crate::world::landmarks::tests::landmarks_of_every_circuit_stand_clear_of_the_road_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn waterfalls_pour_down_a_rock_wall() {
        crate::world::landmarks::tests::waterfalls_pour_down_a_rock_wall_on(&CIRCUITS);
    }
}

mod world_mirror {
    use super::*;

    #[test]
    fn a_desert_canyon_mirrors_its_red_earth_not_grass() {
        crate::world::mirror::tests::a_desert_canyon_mirrors_its_red_earth_not_grass_on(&CIRCUITS);
    }
}

mod world {
    use super::*;

    #[test]
    fn circuits_are_prepared_in_the_menus_and_let_go_for_a_race() {
        crate::world::tests::circuits_are_prepared_in_the_menus_and_let_go_for_a_race_on(&CIRCUITS);
    }

    #[test]
    fn a_circuit_chosen_before_it_is_ready_comes_without_stopping_the_menu() {
        crate::world::tests::a_circuit_chosen_before_it_is_ready_comes_without_stopping_the_menu_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn a_circuit_is_shown_before_its_rest_comes_and_raced_on_whole() {
        crate::world::tests::a_circuit_is_shown_before_its_rest_comes_and_raced_on_whole_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn seen_from_the_road_a_circuit_shows_its_start_first() {
        crate::world::tests::seen_from_the_road_a_circuit_shows_its_start_first_on(&CIRCUITS);
    }

    #[test]
    fn the_looks_at_the_starts_stay_through_a_race() {
        crate::world::tests::the_looks_at_the_starts_stay_through_a_race_on(&CIRCUITS);
    }

    #[test]
    fn seen_from_above_no_circuit_is_looked_at_by_its_start() {
        crate::world::tests::seen_from_above_no_circuit_is_looked_at_by_its_start_on(&CIRCUITS);
    }

    #[test]
    fn the_circuits_nearest_in_the_menu_come_first() {
        crate::world::tests::the_circuits_nearest_in_the_menu_come_first_on(&CIRCUITS);
    }

    #[test]
    fn seen_from_above_the_looks_at_the_starts_go_with_a_race() {
        crate::world::tests::seen_from_above_the_looks_at_the_starts_go_with_a_race_on(&CIRCUITS);
    }
}

mod world_petals {
    use super::*;

    #[test]
    fn petals_drift_over_their_areas_red_and_white() {
        crate::world::petals::tests::petals_drift_over_their_areas_red_and_white_on(&CIRCUITS);
    }
}

mod world_preview {
    use super::*;

    #[test]
    fn a_look_at_the_start_is_quicker_to_work_out_than_the_whole() {
        crate::world::preview::tests::a_look_at_the_start_is_quicker_to_work_out_than_the_whole_on(
            &CIRCUITS,
        );
    }
}

mod world_props {
    use super::*;

    #[test]
    fn no_tree_stands_in_a_dam_s_lake() {
        crate::world::props::tests::no_tree_stands_in_a_dam_s_lake_on(&CIRCUITS);
    }

    #[test]
    fn every_circuit_gets_its_houses() {
        crate::world::props::tests::every_circuit_gets_its_houses_on(&CIRCUITS);
    }
}

mod world_road {
    use super::*;

    #[test]
    fn walls_face_out_and_follow_the_road() {
        crate::world::road::tests::walls_face_out_and_follow_the_road_on(&CIRCUITS);
    }

    #[test]
    fn no_railing_stands_across_another_road() {
        crate::world::road::tests::no_railing_stands_across_another_road_on(&CIRCUITS);
    }

    #[test]
    fn wooden_bridges_have_their_fence_instead_of_walls() {
        crate::world::road::tests::wooden_bridges_have_their_fence_instead_of_walls_on(&CIRCUITS);
    }
}

mod world_sea {
    use super::*;

    #[test]
    fn the_sea_is_blue_the_water_of_a_mine_red_and_a_river_green_grey() {
        crate::world::sea::tests::the_sea_is_blue_the_water_of_a_mine_red_and_a_river_green_grey_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn the_sea_mirrors_but_on_loop_island_and_as_the_options_say() {
        crate::world::sea::tests::the_sea_mirrors_but_on_loop_island_and_as_the_options_say_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn the_mirroring_sea_maps_the_whole_track_and_round_it() {
        crate::world::sea::tests::the_mirroring_sea_maps_the_whole_track_and_round_it_on(&CIRCUITS);
    }

    #[test]
    fn every_circuit_has_shore_near_its_road() {
        crate::world::sea::tests::every_circuit_has_shore_near_its_road_on(&CIRCUITS);
    }
}

mod world_shoulders {
    use super::*;

    #[test]
    fn every_road_of_every_circuit_has_its_shoulders() {
        crate::world::shoulders::tests::every_road_of_every_circuit_has_its_shoulders_on(&CIRCUITS);
    }

    #[test]
    fn tunnels_and_bridges_are_paved_beside_the_road() {
        crate::world::shoulders::tests::tunnels_and_bridges_are_paved_beside_the_road_on(&CIRCUITS);
    }

    #[test]
    fn the_bay_is_mostly_grass_and_the_gold_mine_has_none() {
        crate::world::shoulders::tests::the_bay_is_mostly_grass_and_the_gold_mine_has_none_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn fellowship_road_s_shoulders_are_grass_and_bare_earth_in_patches() {
        crate::world::shoulders::tests::fellowship_road_s_shoulders_are_grass_and_bare_earth_in_patches_on(&CIRCUITS);
    }
}

mod world_structures {
    use super::*;

    #[test]
    fn a_tunnels_top_is_in_flower_where_the_fields_round_it_are() {
        crate::world::structures::tests::a_tunnels_top_is_in_flower_where_the_fields_round_it_are_on(&CIRCUITS);
    }

    #[test]
    fn concrete_bridges_have_a_deck_out_to_the_walls() {
        crate::world::structures::tests::concrete_bridges_have_a_deck_out_to_the_walls_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn a_wooden_bridge_s_boards_show_their_ends_at_each_gap() {
        crate::world::structures::tests::a_wooden_bridge_s_boards_show_their_ends_at_each_gap_on(
            &CIRCUITS,
        );
    }
}

mod world_terrain {
    use super::*;

    #[test]
    fn terrain_hugs_the_road_and_the_track_stays_dry() {
        crate::world::terrain::tests::terrain_hugs_the_road_and_the_track_stays_dry_on(&CIRCUITS);
    }

    #[test]
    fn the_track_edge_is_where_the_earth_rises_inside_the_walls() {
        crate::world::terrain::tests::the_track_edge_is_where_the_earth_rises_inside_the_walls_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn turbo_bay_is_an_island_with_a_river_under_the_bridge() {
        crate::world::terrain::tests::turbo_bay_is_an_island_with_a_river_under_the_bridge_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn turbo_bay_dives_to_the_foot_of_its_lighthouse_in_full_view() {
        crate::world::terrain::tests::turbo_bay_dives_to_the_foot_of_its_lighthouse_in_full_view_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn turbo_bay_passes_over_its_own_tunnel() {
        crate::world::terrain::tests::turbo_bay_passes_over_its_own_tunnel_on(&CIRCUITS);
    }

    #[test]
    fn gold_mine_climbs_its_mountain_and_passes_over_its_galleries() {
        crate::world::terrain::tests::gold_mine_climbs_its_mountain_and_passes_over_its_galleries_on(&CIRCUITS);
    }

    #[test]
    fn gold_mine_s_great_lake_is_the_bottom_of_an_open_pit() {
        crate::world::terrain::tests::gold_mine_s_great_lake_is_the_bottom_of_an_open_pit_on(
            &CIRCUITS,
        );
    }

    #[test]
    fn loop_island_is_an_island_of_checkerboard_cliffs() {
        crate::world::terrain::tests::loop_island_is_an_island_of_checkerboard_cliffs_on(&CIRCUITS);
    }

    #[test]
    fn snow_lies_on_down_the_gullies_of_the_mountains_and_forest_climbs_them() {
        crate::world::terrain::tests::snow_lies_on_down_the_gullies_of_the_mountains_and_forest_climbs_them_on(&CIRCUITS);
    }

    #[test]
    fn the_borders_between_grounds_run_smoothly_not_in_steps() {
        crate::world::terrain::tests::the_borders_between_grounds_run_smoothly_not_in_steps_on(
            &CIRCUITS,
        );
    }
}

mod world_tufts {
    use super::*;

    #[test]
    fn tufts_grow_on_grass_shoulders_only() {
        crate::world::tufts::tests::tufts_grow_on_grass_shoulders_only_on(&CIRCUITS);
    }

    #[test]
    fn the_grass_edge_lies_along_the_asphalt_on_grass_shoulders() {
        crate::world::tufts::tests::the_grass_edge_lies_along_the_asphalt_on_grass_shoulders_on(
            &CIRCUITS,
        );
    }
}

mod bike_model_tyre {
    use super::*;

    #[test]
    fn the_imported_bikes_wear_every_compound() {
        crate::bike::model::tyre::tests::the_imported_bikes_wear_every_compound_on(&ROSTER);
    }
}

/// The Meshy bikes' models, read from their folders.
mod bike_model_imported_game {
    use bevy::prelude::*;

    #[test]
    fn a_model_is_read_from_its_folder_once() {
        let shogun = crate::bike::model::imported::model("shogun").unwrap();
        assert!(
            std::ptr::eq(
                shogun,
                crate::bike::model::imported::model("shogun").unwrap()
            ),
            "once"
        );
        assert_eq!(shogun.geometry.rear_axle, Vec3::new(0.0, 0.313, 0.68));
        assert!(shogun.steers);
        assert_eq!(shogun.tyres, crate::bike::model::tyre::Compound::Medium);
        let legend = crate::bike::model::imported::model("legend").unwrap();
        assert!(!legend.steers, "its fork welded to the fairing");
        assert_eq!(legend.tyres, crate::bike::model::tyre::Compound::Hard);
        assert_ne!(shogun, legend);
        assert!(
            crate::bike::model::imported::models().contains(&shogun)
                && crate::bike::model::imported::models().contains(&legend)
        );
    }
}

/// Feudal Japan's pack of props: a surface for the water to mirror.
mod world_prop_pack_game {
    use bevy::prelude::*;

    use crate::bike::model::imported::{read, texture};
    use crate::world::prop_pack::{bounds, surface};

    /// A pack of props, read as the game reads it.
    fn pack() -> Vec<(String, Mesh)> {
        read(crate::content::file("models/props/props.bin"))
            .meshes
            .into_iter()
            .map(|piece| (piece.name, piece.mesh))
            .collect()
    }

    #[test]
    fn every_prop_has_a_surface_for_the_water_to_mirror_in_its_colours() {
        let atlas = texture(crate::content::file("models/props/color.jpg"), true);
        let mut total = 0;
        for (name, mesh) in pack() {
            let points = surface(&mesh, &atlas);
            let (low, high) = bounds(&mesh);
            assert!(!points.is_empty(), "{name}");
            // Over its faces, inside its box, a few to each square metre at
            // most.
            for point in &points {
                assert!(
                    point.at.cmpge(low - 0.01).all() && point.at.cmple(high + 0.01).all(),
                    "{name}"
                );
                assert!(
                    point.color.iter().all(|c| (0.0..=1.0).contains(c)),
                    "{name}"
                );
            }
            let size = (high - low).max_element();
            assert!(points.len() < 60_000, "{name}: {} points", points.len());
            assert!(
                points.len() as f32 > size,
                "{name}: {} points",
                points.len()
            );
            total += points.len();
        }
        assert!(total < 400_000, "{total}");
        // Not all the atlas's colour is grey: a vermilion torii is red.
        let torii = surface(
            &pack()
                .into_iter()
                .find(|(name, _)| name == "o_torii")
                .unwrap()
                .1,
            &atlas,
        );
        let red = torii
            .iter()
            .filter(|p| p.color[0] > 2.0 * p.color[1])
            .count();
        assert!(red * 3 > torii.len(), "{red} of {}", torii.len());
    }
}
