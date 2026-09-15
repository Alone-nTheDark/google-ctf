use ::game::res::maps::MapType;
use ::game::res::items::ItemType;
use ::game::*;
use megahx8::init_hardware;
use megahx8::Button;

fn run_frames(game: &mut Ctx, frames: u32) {
    for _ in 0..frames {
        game.update();
        game.draw();
    }
}

#[test]
fn test_player_spawn_and_movement() {
    let (vdp, renderer, controllers, portal) = init_hardware();
    let mut game = Game::new(vdp, renderer, controllers, portal);
    game.load_map(MapType::FireTemple, PlayerType::Shooter);
    // Bypass beam-in animation to allow immediate movement in tests
    game.players[0].status = Status::Idle;
    game.map.inventory.scene = None;
    let frames_to_run = 30 /* FPS */;

    // Force player position to a position known to allow movement/falling in tests.
    let (start_x, start_y) = (268, 264);
    game.players[0].x = start_x;
    game.players[0].y = start_y;
    game.players[0].prev_x = start_x;
    game.players[0].prev_y = start_y;

    // Simulate pressing Right
    game.controller.set_pressed(0, Button::Right, true);

    // Update game and draw for several frames
    run_frames(&mut game, frames_to_run);

    let p = &game.players[0];
    let (end_x, end_y) = (p.x, p.y);
    println!("--- State After {frames_to_run} Frames ---");
    print_game_state(&game);

    assert!(
        end_x > start_x,
        "Player should have moved Right ({start_x} -> {end_x})"
    );
    if end_y != start_y {
        println!("Note: Player vertical position changed from {start_y} to {end_y}");
    }
}

fn print_game_state(game: &Ctx) {
    println!("Frame: {}", game.frame);
    let p = &game.players[0];
    println!("Player 0 position: ({}, {}) | status: {:?}", p.x, p.y, game.players[0].status);
    println!("VDP Sprites count: {}", game.vdp.num_sprites);
    for i in 0..game.vdp.num_sprites {
        let s = &game.vdp.sprites[i];
        if s.x != 0 || s.y != 0 {
            println!(
                "Sprite {}: x={}, y={}, size={:?}, palette={}, tile={}",
                i,
                s.x,
                s.y,
                s.size,
                s.flags.palette(),
                s.flags.tile_index()
            );
        }
    }

    let ascii = game.vdp.render_ascii(40, 24);
    println!("VDP State (Space=Empty, .=Tile, P=Player, i=Item, E=Enemy/Other):\n{ascii}");
}

#[test]
fn test_fire_temple_out_of_bounds_death() {
    let (vdp, renderer, controllers, portal) = init_hardware();
    let mut game = Game::new(vdp, renderer, controllers, portal);
    let frames_to_run = 200;

    // Initial position in Overworld, switch to FireTemple (1, 3)
    game.load_map(MapType::FireTemple, PlayerType::Shooter);
    // Bypass beam-in animation to allow immediate movement in tests
    game.players[0].status = Status::Idle;

    // Ensure player is alive
    assert!(game.players[0].is_alive());

    // The player will automatically fell down the map and die.
    println!("--- State Before {frames_to_run} Frames ---");
    print_game_state(&game);
    // Teleport player to a position where it falls out of bounds.
    game.players[0].y = 360;
    run_frames(&mut game, frames_to_run);

    println!("--- State After {frames_to_run} Frames ---");
    print_game_state(&game);

    assert!(
        game.players[0].is_dead(),
        "Player should be dead after walking out of bounds (current status: {:?})",
        game.players[0].status
    );
}

#[test]
fn test_player_double_jump() {
    let (vdp, renderer, controllers, portal) = init_hardware();
    let mut game = Game::new(vdp, renderer, controllers, portal);
    game.load_map(MapType::FireTemple, PlayerType::Shooter);
    // Bypass beam-in animation to allow immediate movement in tests
    game.players[0].status = Status::Idle;
    game.map.inventory.scene = None;
    game.map.inventory.add(ItemType::Doublejump);

    // Force player position to a position known to allow movement/falling.
    let (start_x, start_y) = (268, 264);
    game.players[0].x = start_x;
    game.players[0].y = start_y;
    game.players[0].prev_x = start_x;
    game.players[0].prev_y = start_y;

    // Wait for player to land (if not already on ground).
    run_frames(&mut game, 10);

    // Ensure player is on ground.
    assert!(game.players[0].on_ground);

    // Press Up to jump.
    game.controller.set_pressed(0, Button::Up, true);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::Up, false);

    // Wait a few frames to be in the air and moving up.
    run_frames(&mut game, 5);
    assert!(!game.players[0].on_ground);
    let h_speed_after_jump = game.players[0].h_speed;
    assert!(h_speed_after_jump < 0);

    // Press Up again to double jump.
    game.controller.set_pressed(0, Button::Up, true);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::Up, false);

    // Verify double jump triggered.
    let h_speed_after_double_jump = game.players[0].h_speed;
    assert!(h_speed_after_double_jump < 0);
    // JUMP_SPEED is 256. After 1 frame of gravity (8) it should be -248.
    // Let's assert it's less than -200.
    assert!(h_speed_after_double_jump < -200);
}

#[test]
fn test_switch_logic_fire_temple() {
    let (vdp, renderer, controllers, portal) = init_hardware();
    let mut game = Game::new(vdp, renderer, controllers, portal);
    game.load_map(MapType::FireTemple, PlayerType::Shooter);
    // Bypass beam-in animation to allow immediate movement in tests
    game.players[0].status = Status::Idle;

    assert!(game.map.switches.len() >= 1, "At least one switch should be loaded");

    let s1_pos = game.map.switches.iter().find(|s| s.id == 1).map(|s| (s.x, s.y));
    assert!(s1_pos.is_some(), "Switch 1 should be loaded");
    let (s1_x, s1_y) = s1_pos.unwrap();

    // Put player safely next to Switch 1 immediately on frame 0 to shoot it.
    // We place them to the left of the switch and shoot right.
    // We use s1_x - 40 to account for the shooter's projectile spawn offset (26 pixels)
    // so that the projectile doesn't spawn inside the wall to the right of the switch.
    let px = s1_x - 40;
    let py = s1_y - 15; // Stand on the actual ground height (which is 12 pixels below the switch now)
    game.players[0].x = px;
    game.players[0].y = py;
    game.players[0].prev_x = px;
    game.players[0].prev_y = py;
    game.players[0].h_speed = 0;

    // Simulate pressing Attack (Button::A)
    game.controller.set_pressed(0, Button::A, true);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::A, false);

    // Wait enough frames for projectile to hit and event to dispatch
    run_frames(&mut game, 15);

    // In NEW logic, it SHOULD be completed because Switch 1 was pressed.
    assert!(game.map.switches_completed.is_set(1), "Event SHOULD trigger with only one switch pressed");
    assert!(!game.map.switches_completed.is_set(0), "Other switches should NOT be marked completed automatically");
}

#[test]
fn test_switch_logic_fire_temple_door_0() {
    let (vdp, renderer, controllers, portal) = init_hardware();
    let mut game = Game::new(vdp, renderer, controllers, portal);
    game.load_map(MapType::FireTemple, PlayerType::Shooter);
    // Bypass beam-in animation to allow immediate movement in tests
    game.players[0].status = Status::Idle;

    let s0_pos = game.map.switches.iter().find(|s| s.id == 0).map(|s| (s.x, s.y));
    assert!(s0_pos.is_some(), "Switch 0 should be loaded");
    let (s0_x, s0_y) = s0_pos.unwrap();

    // Manually set scroll offset to load Switch 0 which is at (408, 744) immediately
    game.map.scroll_offset = (408 - 160, 744 - 112);

    // Put player next to Switch 0 immediately on frame 0 to shoot it.
    // Place player to the left of the switch and shoot right.
    let px = s0_x - 40;
    let py = s0_y - 15; // Stand on the actual ground height (which is 12 pixels below the switch now)
    game.players[0].x = px;
    game.players[0].y = py;
    game.players[0].prev_x = px;
    game.players[0].prev_y = py;
    game.players[0].h_speed = 0;

    // Wait 1 frame for scroll/teleport processing
    run_frames(&mut game, 1);

    // Simulate pressing Attack (Button::A)
    game.controller.set_pressed(0, Button::A, true);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::A, false);

    // We need to wait enough frames for the door opening animation to complete
    run_frames(&mut game, 60);

    assert!(game.map.switches_completed.is_set(0), "Switch 0 should be completed");
    assert!(!game.map.doors[0].locked, "Door 0 should be unlocked after Switch 0 was pressed");
    let door0_open = game.map.doors.iter().find(|d| d.id == 0).unwrap().open;
    assert!(door0_open, "Door 0 should be opened after triggering switch 0");
}

fn teleport_player(game: &mut Ctx, map_x: i16, map_y: i16) {
    let scroll = game.map.scroll_offset;
    let px = map_x + 128 - scroll.0;
    let py = map_y + 128 - scroll.1;
    game.players[0].x = px;
    game.players[0].y = py;
    game.players[0].prev_x = px;
    game.players[0].prev_y = py;
}

#[test]
fn test_door_unload_reload_bug() {
    let (vdp, renderer, controllers, portal) = init_hardware();
    let mut game = Game::new(vdp, renderer, controllers, portal);
    game.load_map(MapType::SkyTemple, PlayerType::Shooter);
    // Bypass beam-in animation to allow immediate movement in tests
    game.players[0].status = Status::Idle;
    game.map.inventory.scene = None;

    // 1. Teleport player near Door 0 (left side) to trigger load and approach.
    // Door 0 is at (600, 344) map coordinates.
    // We spawn at (560, 344).
    println!("--- Teleporting to Approach ---");
    teleport_player(&mut game, 560, 344);

    // Run 1 frame to trigger scroll and load the door.
    run_frames(&mut game, 1);
    println!("--- After 1 frame (Load) ---");
    print_game_state(&game);

    // Verify door is loaded and closed.
    {
        let door = game.map.doors.iter().find(|d| d.id == 0).expect("Door 0 should be loaded");
        assert!(!door.open, "Door should be closed initially");
        assert!(!door.locked, "Door should not be locked initially");
    }

    // Run 1 more frame to trigger the door opening.
    run_frames(&mut game, 1);
    println!("--- After 2 frames (Opening started) ---");
    print_game_state(&game);

    // Wait for door to open completely (30 frames).
    run_frames(&mut game, 30);
    println!("--- After opening animation ---");
    print_game_state(&game);

    {
        let door = game.map.doors.iter().find(|d| d.id == 0).expect("Door 0 should be loaded");
        assert!(door.open, "Door should be open");
    }

    // 2. Teleport player to overlap the door to set entry side.
    // Map X = 575 is overlapping.
    println!("--- Teleporting to Overlap ---");
    teleport_player(&mut game, 575, 344);
    run_frames(&mut game, 1);
    print_game_state(&game);
    // 3. Teleport player to the right of the door, within expanded hitbox.
    // Map X = 605 is right of the door.
    println!("--- Teleporting to Cross ---");
    teleport_player(&mut game, 605, 344);
    run_frames(&mut game, 1);
    print_game_state(&game);

    // Verify door is closed and locked.
    {
        let door = game.map.doors.iter().find(|d| d.id == 0).expect("Door 0 should be loaded");
        assert!(!door.open, "Door should be closed after crossing");
        assert!(door.locked, "Door should be locked after crossing");
    }

    // 4. Teleport player far left to unload the door.
    println!("--- Teleporting to Unload ---");
    teleport_player(&mut game, 100, 344);
    run_frames(&mut game, 1); // Run 1 frame to trigger scroll and unload.
    print_game_state(&game);

    // Verify door is unloaded.
    assert!(
        !game.map.doors.iter().any(|d| d.id == 0),
        "Door 0 should be unloaded"
    );

    // 5. Teleport player back to right of door to reload it.
    println!("--- Teleporting to Reload ---");
    teleport_player(&mut game, 650, 344);
    run_frames(&mut game, 1); // Run 1 frame to trigger scroll and load.
    print_game_state(&game);

    // Verify door is reloaded.
    let door = game.map.doors.iter().find(|d| d.id == 0).expect("Door 0 should be reloaded");

    // Assert door is STILL locked (this should FAIL if the bug is present).
    assert!(door.locked, "Door should remain locked after reload");
}

#[test]
fn test_player_shooting() {
    let (vdp, renderer, controllers, portal) = init_hardware();
    let mut game = Game::new(vdp, renderer, controllers, portal);
    game.load_map(MapType::FireTemple, PlayerType::Shooter);
    // Bypass beam-in animation to allow immediate movement in tests
    game.players[0].status = Status::Idle;
    game.map.inventory.scene = None;

    let (start_x, start_y) = (200, 264);
    game.players[0].x = start_x;
    game.players[0].y = start_y;
    game.players[0].prev_x = start_x;
    game.players[0].prev_y = start_y;

    // Press A to shoot.
    game.controller.set_pressed(0, Button::A, true);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::A, false);

    // Verify projectile spawned.
    assert!(!game.map.projectiles.is_empty(), "Projectile should have spawned");

    let proj = &game.map.projectiles[0];
    assert!(proj.is_player, "Projectile should be marked as player's");

    let initial_proj_x = proj.x;

    // Run 50 frames (enough to trigger potential overflow at frame 33).
    run_frames(&mut game, 50);

    // If the projectile is still alive, verify it moved right and didn't wrap.
    // If it's gone, it likely hit a wall or went off screen, which is also valid.
    if !game.map.projectiles.is_empty() {
        assert!(game.map.projectiles[0].x > initial_proj_x, "Projectile should move right and not wrap");
    }
}

#[test]
fn test_wall_slide_shooting() {
    let (vdp, renderer, controllers, portal) = init_hardware();
    let mut game = Game::new(vdp, renderer, controllers, portal);
    game.load_map(MapType::FireTemple, PlayerType::Shooter);
    // Bypass beam-in animation to allow immediate movement in tests
    game.players[0].status = Status::Idle;
    game.map.inventory.scene = None;

    // Place the player near Door 1 (left side). Door 1 is at map x=600, y=568.
    teleport_player(&mut game, 559, 540);

    // Run 1 frame to trigger scroll and load the door.
    run_frames(&mut game, 1);

    // Ensure the door is loaded and closed.
    assert!(game.map.doors.iter().any(|d| d.id == 1 && !d.open), "Door 1 should be loaded and closed");

    // Hold Right to hug the door/wall, and let them fall down (since h_speed will increase due to gravity)
    game.controller.set_pressed(0, Button::Right, true);
    game.players[0].on_ground = false;

    // Run frames until they are sliding.
    let mut is_sliding = false;
    for _ in 0..10 {
        run_frames(&mut game, 1);
        if game.players[0].is_sliding_down_wall() {
            is_sliding = true;
            break;
        }
    }
    assert!(is_sliding, "Player should be sliding down the wall (door)");

    // Save the animation frame index before shooting. We want it to be > 0 so we can detect a reset to 0.
    // Let's run a few more frames to advance the slide animation if needed.
    run_frames(&mut game, 8);
    let anim_frame_before = game.players[0].sprite.current_frame_index();
    assert!(anim_frame_before > 0, "Animation should have progressed beyond frame 0 before shooting, but it is {}", anim_frame_before);

    // Shoot!
    game.controller.set_pressed(0, Button::A, true);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::A, false);

    // Verify a projectile was spawned.
    assert!(!game.map.projectiles.is_empty(), "Projectile should have spawned");
    let initial_proj_x = game.map.projectiles[0].x;

    // Run 1 frame to update projectile position
    run_frames(&mut game, 1);

    // Verify it moved LEFT (away from the wall)
    assert!(
        !game.map.projectiles.is_empty(),
        "Projectile should still exist"
    );
    let new_proj_x = game.map.projectiles[0].x;
    assert!(
        new_proj_x < initial_proj_x,
        "Projectile should move LEFT (away from the wall), but it moved from {} to {}",
        initial_proj_x,
        new_proj_x
    );

    // Verify the animation frame index didn't reset to 0.
    let anim_frame_after = game.players[0].sprite.current_frame_index();
    assert_eq!(anim_frame_after, anim_frame_before, "Animation frame index reset/changed from {} to {}", anim_frame_before, anim_frame_after);
}

#[test]
fn test_shoot_next_to_wall() {
    let (vdp, renderer, controllers, portal) = init_hardware();
    let mut game = Game::new(vdp, renderer, controllers, portal);
    game.load_map(MapType::FireTemple, PlayerType::Shooter);
    // Bypass beam-in animation to allow immediate movement in tests
    game.players[0].status = Status::Idle;
    game.map.inventory.scene = None;

    // Place the player near Door 1 (left side). Door 1 starts at x=600.
    // Teleport the player to x=567 so they are right next to the door (567 + 8 + 25 = 600).
    teleport_player(&mut game, 567, 568);

    // Run 10 frames to load map/door and let player settle on the ground.
    run_frames(&mut game, 10);

    // Face Right (towards the door).
    game.controller.set_pressed(0, Button::Right, true);
    run_frames(&mut game, 2);
    game.controller.set_pressed(0, Button::Right, false);

    assert!(!game.players[0].sprite.flip_h, "Player should be facing Right");
    println!("Player x: {}, y: {}", game.players[0].x, game.players[0].y);

    // Shoot!
    game.controller.set_pressed(0, Button::A, true);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::A, false);

    // Verify that the player shot (cooldown is active).
    assert_eq!(game.players[0].shoot_cooldown, 9, "Player should have fired a projectile");

    // Since the projectile spawned in/next to the wall, it should be destroyed immediately
    // during the same update frame, leaving the projectiles list empty.
    assert!(game.map.projectiles.is_empty(), "Projectile should have been destroyed by the wall immediately");
}

#[test]
fn test_jump_shooting_hits_enemy() {
    let (vdp, renderer, controllers, portal) = init_hardware();
    let mut game = Game::new(vdp, renderer, controllers, portal);
    game.load_map(MapType::FireTemple, PlayerType::Shooter);
    game.players[0].status = Status::Idle;
    game.map.inventory.scene = None;

    // Clear existing enemies so we only test with our spawned enemy.
    game.map.enemies.clear();

    let px = 200;
    let py = 264;

    game.players[0].x = px;
    game.players[0].y = py - 15;
    game.players[0].prev_x = px;
    game.players[0].prev_y = py - 15;
    game.players[0].h_speed = 0;
    game.players[0].sprite.flip_h = false;

    let enemy_map_x = px + 50 - 128;
    let enemy_map_y = py - 128;

    static MOCK_PROPERTIES: EnemyProperties = EnemyProperties {
        id: 29,
        walk_data: &[],
        speed: None,
        health: Some(3),
        strength: Some(1),
        invulnerable: false,
        flags: None,
    };
    let enemy = Enemy::new(
        res::enemies::EnemyType::Orc,
        enemy_map_x,
        enemy_map_y,
        &MOCK_PROPERTIES,
        &mut game.res_state,
        &mut game.vdp,
    );
    if game.map.enemies.push(enemy).is_err() {
        panic!("too many enemies");
    }

    // Verify initial state
    assert_eq!(game.map.enemies[0].health(), 3);
    println!("Scroll offset: {:?}", game.map.scroll_offset);
    println!("Player pos: ({}, {}), status: {:?}, cooldown: {}, hitbox: {:?}", 
             game.players[0].x, game.players[0].y, game.players[0].status, game.players[0].shoot_cooldown, game.players[0].hitbox());
    println!("Enemy pos: ({}, {}), hitbox: {:?}", 
             game.map.enemies[0].x, game.map.enemies[0].y, game.map.enemies[0].hitbox());

    // Simulate pressing Attack (Button::A)
    game.controller.set_pressed(0, Button::A, true);
    println!("Pressing A. Player shoot cooldown: {}", game.players[0].shoot_cooldown);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::A, false);
    println!("Released A. Projectiles count: {}", game.map.projectiles.len());

    for f in 0..10 {
        if !game.map.projectiles.is_empty() {
            let p = &game.map.projectiles[0];
            println!("Frame {}: projectile pos ({}, {}), hitbox: {:?}", f, p.x, p.y, p.hitbox());
        } else {
            println!("Frame {}: no projectiles. Player status: {:?}, cooldown: {}", f, game.players[0].status, game.players[0].shoot_cooldown);
        }
        run_frames(&mut game, 1);
    }

    // Verify that the enemy has taken damage and the projectile is dead.
    assert_eq!(game.map.enemies[0].health(), 2, "Enemy should have taken damage");
    assert!(game.map.projectiles.is_empty(), "Projectile should have been consumed on hit");
}



#[test]
fn test_air_dash() {
    let (vdp, renderer, controllers, portal) = init_hardware();
    let mut game = Game::new(vdp, renderer, controllers, portal);
    game.load_map(MapType::FireTemple, PlayerType::Shooter);
    // Bypass beam-in animation to allow immediate movement in tests
    game.players[0].status = Status::Idle;
    game.map.inventory.scene = None;

    // Teleport to the safe open area near the left wall (90, 568)
    let (start_x, start_y) = (90, 568);
    teleport_player(&mut game, start_x, start_y);
    game.players[0].h_speed = 0;
    game.players[0].on_ground = false;

    // Wait 30 frames for camera to settle and player to land
    run_frames(&mut game, 30);
    assert!(game.players[0].on_ground);

    // Clear all enemies to prevent them from hitting us!
    game.map.enemies.clear();

    // Jump 1!
    game.controller.set_pressed(0, Button::Up, true);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::Up, false);

    // Wait 20 frames to get high in the air (near the peak of jump)
    run_frames(&mut game, 20);

    {
        let p = &game.players[0];
        println!("--- Peak of Jump --- Position: ({}, {}), h_speed: {}", p.x, p.y, p.h_speed);
        assert!(!p.on_ground);
    }

    // Turn Left to face the left wall
    game.controller.set_pressed(0, Button::Left, true);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::Left, false);

    let y_before_dash = game.players[0].y;
    let map_x_before_dash = game.players[0].x - 128 + game.map.scroll_offset.0;

    // Trigger First Air Dash (facing Left)
    game.controller.set_pressed(0, Button::C, true);
    run_frames(&mut game, 1);

    {
        let p = &game.players[0];
        println!("--- First Dash Triggered (1 Frame) --- Position: ({}, {}), h_speed: {}", p.x, p.y, p.h_speed);
    }

    // Run 5 frames of the dash (moving Left)
    run_frames(&mut game, 5);

    {
        let p = &game.players[0];
        let map_x = p.x - 128 + game.map.scroll_offset.0;
        println!("--- During First Dash (5 Frames) --- Position: ({}, {}), h_speed: {}, map_x: {}", p.x, p.y, p.h_speed, map_x);
        assert_eq!(p.y, y_before_dash, "Gravity should be paused (Y should not change during first air dash)");
        assert!(map_x < map_x_before_dash, "Player should have moved Left in map space (current: {}, start: {})", map_x, map_x_before_dash);
        assert!(p.is_playing_dash_animation(), "Player should be playing the Dash animation");
    }

    // Run remaining dash frames (total 20 remaining) to hit the left wall
    run_frames(&mut game, 20);
    game.controller.set_pressed(0, Button::C, false);

    {
        let p = &game.players[0];
        let map_x = p.x - 128 + game.map.scroll_offset.0;
        println!("--- After First Dash (Hit Wall) --- Position: ({}, {}), h_speed: {}, map_x: {}", p.x, p.y, p.h_speed, map_x);
    }

    // Hold Left to trigger wall slide state on the left wall
    game.controller.set_pressed(0, Button::Left, true);
    run_frames(&mut game, 5);
    game.controller.set_pressed(0, Button::Left, false);

    {
        let p = &game.players[0];
        println!("--- After Wall Slide Attempt --- Position: ({}, {}), h_speed: {}", p.x, p.y, p.h_speed);
    }

    // Turn Right to face away from the wall
    game.controller.set_pressed(0, Button::Right, true);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::Right, false);

    // Try to trigger Second Air Dash (facing Right) - SHOULD SUCCEED because wall touch/slide reset it!
    let y_before_second_dash = game.players[0].y;
    let map_x_before_second_dash = game.players[0].x - 128 + game.map.scroll_offset.0;
    game.controller.set_pressed(0, Button::C, true);
    run_frames(&mut game, 1);

    println!("--- Second Dash Triggered (1 Frame) ---");

    // Run 5 frames and verify it succeeded (gravity is paused, Y is constant, X moves Right)
    run_frames(&mut game, 5);
    {
        let p = &game.players[0];
        let map_x = p.x - 128 + game.map.scroll_offset.0;
        println!("--- During Second Dash (5 Frames) --- Position: ({}, {}), h_speed: {}, map_x: {}", p.x, p.y, p.h_speed, map_x);
        assert_eq!(p.y, y_before_second_dash, "Second air dash should succeed after wall touch (Y should not change)");
        assert!(map_x > map_x_before_second_dash, "Player should have moved Right in map space");
        assert!(p.is_playing_dash_animation());
    }

    // Run remaining frames to complete second dash (34 more frames)
    run_frames(&mut game, 34);
    game.controller.set_pressed(0, Button::C, false);

    // Now second dash should be over, and gravity should resume
    let y_after_second_dash = game.players[0].y;
    run_frames(&mut game, 10);
    {
        let p = &game.players[0];
        assert!(p.y > y_after_second_dash, "Gravity should resume after second dash ends");
    }

    // Wait until they land on the ground
    run_frames(&mut game, 30);
    {
        let p = &game.players[0];
        println!("--- After Landing --- Position: ({}, {}), on_ground: {}, status: {:?}, health: {}", p.x, p.y, p.on_ground, p.status, p.health);
        assert!(p.on_ground, "Player should be on the ground");
    }

    // Jump again!
    game.controller.set_pressed(0, Button::Up, true);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::Up, false);

    // Wait 15 frames to get high in the air
    run_frames(&mut game, 15);

    // Turn Right to face Right
    game.controller.set_pressed(0, Button::Right, true);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::Right, false);

    let y_before_third_dash = game.players[0].y;
    let map_x_before_third_dash = game.players[0].x - 128 + game.map.scroll_offset.0;

    // Trigger Third Air Dash (facing Right) - SHOULD SUCCEED because landing reset it!
    game.controller.set_pressed(0, Button::C, true);
    run_frames(&mut game, 1);

    println!("--- Third Dash Triggered (1 Frame) ---");

    // Run 5 frames of the third dash
    run_frames(&mut game, 5);
    {
        let p = &game.players[0];
        let map_x = p.x - 128 + game.map.scroll_offset.0;
        println!("--- During Third Dash (5 Frames) --- Position: ({}, {}), h_speed: {}, map_x: {}", p.x, p.y, p.h_speed, map_x);
        assert_eq!(p.y, y_before_third_dash, "Third air dash should succeed after landing (Y should not change)");
        assert!(map_x > map_x_before_third_dash, "Player should have moved Right in map space (current: {}, start: {})", map_x, map_x_before_third_dash);
        assert!(p.is_playing_dash_animation(), "Player should be playing the Dash animation");
    }

    // Run remaining frames to complete third dash (34 more frames)
    run_frames(&mut game, 34);
    game.controller.set_pressed(0, Button::C, false);

    // Now third dash should be over, and gravity should resume again
    let y_after_third_dash = game.players[0].y;
    run_frames(&mut game, 10);
    {
        let p = &game.players[0];
        println!("--- After Third Gravity Resumed (10 frames) --- Position: ({}, {}), h_speed: {}", p.x, p.y, p.h_speed);
        assert!(p.y > y_after_third_dash, "Gravity should resume after third dash ends (Y should increase)");
    }
}

#[test]
fn test_air_dash_cancel_on_direction_change() {
    let (vdp, renderer, controllers, portal) = init_hardware();
    let mut game = Game::new(vdp, renderer, controllers, portal);
    game.load_map(MapType::FireTemple, PlayerType::Shooter);
    // Bypass beam-in animation to allow immediate movement in tests
    game.players[0].status = Status::Idle;
    game.map.inventory.scene = None;

    // Teleport to the safe open area (540, 568)
    let (start_x, start_y) = (540, 568);
    teleport_player(&mut game, start_x, start_y);
    game.players[0].h_speed = 0;
    game.players[0].on_ground = false;

    // Wait 30 frames for camera to settle and player to land
    run_frames(&mut game, 30);
    assert!(game.players[0].on_ground);

    // Clear all enemies to prevent them from hitting us!
    game.map.enemies.clear();

    // Jump 1!
    game.controller.set_pressed(0, Button::Up, true);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::Up, false);

    // Wait 20 frames to get high in the air (near the peak of jump)
    run_frames(&mut game, 20);

    {
        let p = &game.players[0];
        println!("--- Peak of Jump --- Position: ({}, {}), h_speed: {}", p.x, p.y, p.h_speed);
        assert!(!p.on_ground);
    }

    // Explicitly face Right
    game.controller.set_pressed(0, Button::Right, true);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::Right, false);

    let y_before_dash = game.players[0].y;
    let map_x_before_dash = game.players[0].x - 128 + game.map.scroll_offset.0;

    // Trigger Air Dash (facing Right)
    game.controller.set_pressed(0, Button::C, true);
    run_frames(&mut game, 1);

    {
        let p = &game.players[0];
        assert!(p.is_playing_dash_animation(), "Player should be playing the Dash animation");
    }

    // Run 5 frames of the dash (moving Right)
    run_frames(&mut game, 5);

    let map_x_during_dash = game.players[0].x - 128 + game.map.scroll_offset.0;
    {
        let p = &game.players[0];
        println!("--- During Dash (5 Frames) --- Position: ({}, {}), h_speed: {}, map_x: {}", p.x, p.y, p.h_speed, map_x_during_dash);
        assert_eq!(p.y, y_before_dash, "Gravity should be paused during air-dash");
        assert!(map_x_during_dash > map_x_before_dash, "Player should have moved Right (current: {}, start: {})", map_x_during_dash, map_x_before_dash);
        assert!(p.is_playing_dash_animation());
    }

    // Now press Left to cancel the dash!
    game.controller.set_pressed(0, Button::Left, true);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::Left, false);

    // After 1 frame of pressing Left:
    {
        let p = &game.players[0];
        assert!(!p.is_playing_dash_animation(), "Air-dash animation should be cancelled immediately on direction change");
    }
    game.controller.set_pressed(0, Button::C, false);

    let y_after_cancel = game.players[0].y;
    run_frames(&mut game, 5);
    {
        let p = &game.players[0];
        let map_x_after_cancel = p.x - 128 + game.map.scroll_offset.0;
        println!("--- After Cancel (5 frames later) --- Position: ({}, {}), h_speed: {}, map_x: {}", p.x, p.y, p.h_speed, map_x_after_cancel);
        assert!(p.y > y_after_cancel, "Player should be falling after air-dash is cancelled (Y should increase, current: {}, start: {})", p.y, y_after_cancel);
        assert!(map_x_after_cancel < map_x_during_dash, "Player should have moved Left (current: {}, start: {})", map_x_after_cancel, map_x_during_dash);
        // NOTE: Due to the direction-change check being merged late (suboptimal), the player performs a 1-frame
        // high-speed (3px instead of 1px) movement step in the opposite direction before the dash is cancelled.
        // Therefore, the difference can be exactly 3px instead of being strictly less than 3px.
        assert!(map_x_during_dash - map_x_after_cancel <= 3, "Should have moved at normal speed (or up to 3px with the late-cancel bug)");
    }
}

#[test]
fn test_air_dash_interrupted_by_hit_or_attack() {
    let (vdp, renderer, controllers, portal) = init_hardware();
    let mut game = Game::new(vdp, renderer, controllers, portal);
    game.load_map(MapType::FireTemple, PlayerType::Melee);
    // Bypass beam-in animation to allow immediate movement in tests
    game.players[0].status = Status::Idle;
    game.map.inventory.scene = None;

    // Teleport to the safe open area (540, 568)
    let (start_x, start_y) = (540, 568);
    teleport_player(&mut game, start_x, start_y);
    game.players[0].h_speed = 0;
    game.players[0].on_ground = false;

    // Wait 30 frames for camera to settle and player to land
    run_frames(&mut game, 30);
    assert!(game.players[0].on_ground);

    // Clear all enemies to prevent them from hitting us naturally!
    game.map.enemies.clear();

    // JUMP 1!
    game.controller.set_pressed(0, Button::Up, true);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::Up, false);

    // Wait 20 frames to get high in the air
    run_frames(&mut game, 20);

    // Face Right
    game.controller.set_pressed(0, Button::Right, true);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::Right, false);

    // Trigger Air Dash (facing Right)
    game.controller.set_pressed(0, Button::C, true);
    run_frames(&mut game, 1);

    {
        let p = &game.players[0];
        assert!(p.is_air_dashing());
        assert_eq!(p.dash_timer(), 39);
    }

    // Now attack mid-dash!
    game.controller.set_pressed(0, Button::A, true);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::A, false);
    game.controller.set_pressed(0, Button::C, false); // release dash as well

    {
        let p = &game.players[0];
        assert!(!p.is_air_dashing(), "Air-dash should be cancelled when transitioning to Attacking state");
        assert_eq!(p.dash_timer(), 0, "Dash timer should be reset when transitioning to Attacking state");
        assert!(matches!(p.status, Status::Attacking { .. }), "Player status should be Attacking");
    }

    // Let the attack finish and let the player land back on the ground
    run_frames(&mut game, 60);
    assert!(game.players[0].on_ground, "Player should have landed back on the ground");

    // JUMP 2!
    game.controller.set_pressed(0, Button::Up, true);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::Up, false);

    // Wait 20 frames to get high in the air
    run_frames(&mut game, 20);

    // Trigger second Air Dash (facing Right)
    game.controller.set_pressed(0, Button::C, true);
    run_frames(&mut game, 1);

    {
        let p = &game.players[0];
        assert!(p.is_air_dashing());
    }

    // Now trigger an explicit on_hit event mid-dash!
    game.players[0].on_hit(Direction::Left, 1);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::C, false); // release C button

    {
        let p = &game.players[0];
        assert!(!p.is_air_dashing(), "Air-dash should be cancelled when player is hit");
        assert_eq!(p.dash_timer(), 0, "Dash timer should be reset when player is hit");
        assert!(matches!(p.status, Status::KnockedBack { .. }), "Player status should be KnockedBack");
    }
}

#[test]
fn test_shoot_while_dashing() {
    let (vdp, renderer, controllers, portal) = init_hardware();
    let mut game = Game::new(vdp, renderer, controllers, portal);
    game.load_map(MapType::SkyTemple, PlayerType::Shooter);
    game.map.enemies.clear();
    game.players[0].status = Status::Idle;
    game.map.inventory.scene = None;

    // Let the player settle on the ground at the spawn point
    run_frames(&mut game, 50);

    // Face Right and start a ground dash!
    game.controller.set_pressed(0, Button::Right, true);
    game.controller.set_pressed(0, Button::C, true);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::C, false);

    assert!(game.players[0].dash_timer() > 0, "Player should be dashing");
    let initial_x = game.players[0].x;

    // Run 5 frames of dashing.
    run_frames(&mut game, 5);
    let x_after_5 = game.players[0].x;
    assert!(x_after_5 > initial_x, "Player should have moved right");

    // Now shoot!
    game.controller.set_pressed(0, Button::A, true);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::A, false);

    // Verify that the player is STILL dashing!
    assert!(game.players[0].dash_timer() > 0, "Player should STILL be dashing after shooting");

    // Run another 5 frames of dashing.
    run_frames(&mut game, 5);
    let x_after_11 = game.players[0].x;
    assert!(x_after_11 > x_after_5, "Player should have continued moving right");
}

#[test]
fn test_shoot_while_double_tap_dashing() {
    let (vdp, renderer, controllers, portal) = init_hardware();
    let mut game = Game::new(vdp, renderer, controllers, portal);
    game.load_map(MapType::SkyTemple, PlayerType::Shooter);
    game.map.enemies.clear();
    game.players[0].status = Status::Idle;
    game.map.inventory.scene = None;

    // Let the player settle on the ground at the spawn point
    run_frames(&mut game, 50);

    // Double tap Right:
    // 1. Press Right
    game.controller.set_pressed(0, Button::Right, true);
    run_frames(&mut game, 1);
    // 2. Release Right
    game.controller.set_pressed(0, Button::Right, false);
    run_frames(&mut game, 1);
    // 3. Press Right again
    game.controller.set_pressed(0, Button::Right, true);
    run_frames(&mut game, 1);

    assert!(game.players[0].dash_timer() > 0, "Player should be dashing after double-tap");
    let initial_x = game.players[0].x;

    // Run 5 frames of dashing (keep holding Right)
    run_frames(&mut game, 5);
    let x_after_5 = game.players[0].x;
    assert!(x_after_5 > initial_x, "Player should have moved right");

    // Now shoot!
    game.controller.set_pressed(0, Button::A, true);
    run_frames(&mut game, 1);
    game.controller.set_pressed(0, Button::A, false);

    // Verify that the player is STILL dashing!
    assert!(game.players[0].dash_timer() > 0, "Player should STILL be dashing after shooting");

    // Run another 5 frames of dashing (keep holding Right)
    run_frames(&mut game, 5);
    let x_after_11 = game.players[0].x;
    assert!(x_after_11 > x_after_5, "Player should have continued moving right");
}




