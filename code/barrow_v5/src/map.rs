use std::cmp::Ordering;
use rltk::{ RGB, Rltk, RandomNumberGenerator, BaseMap, Algorithm2D, Point, FastNoise};
use super::{Rect, Position};
use std::cmp::{max, min};
use specs::prelude::*;

const MAPWIDTH : usize = 50;
const MAPHEIGHT : usize = 43;
const MAPCOUNT : usize = MAPHEIGHT * MAPWIDTH;

#[derive(PartialEq, Copy, Clone, Debug)]
pub enum TileType {
    Wall, Floor, StairsUp, StairsDown
}

#[derive(Default, Clone)]
pub struct Map {
    pub tiles : Vec<TileType>,
    pub rooms : Vec<Rect>,
    pub width : i32,
    pub height : i32,
    pub revealed_tiles : Vec<bool>,
    pub visible_tiles : Vec<bool>,
    pub blocked : Vec<bool>,
    pub tile_content : Vec<Vec<Entity>>,
    pub noise_seed : u64,
    pub frame_count : u64
}

impl Map {
    pub fn xy_idx(&self, x: i32, y: i32) -> usize {
        (y as usize * self.width as usize) + x as usize
    }

    fn apply_room_to_map(&mut self, room : &Rect) {
        for y in room.y1 +1 ..= room.y2 {
            for x in room.x1 + 1 ..= room.x2 {
                let idx = self.xy_idx(x, y);
                self.tiles[idx] = TileType::Floor;
            }
        }
    }

    fn apply_horizontal_tunnel(&mut self, x1:i32, x2:i32, y:i32) {
        for x in min(x1,x2) ..= max(x1,x2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width as usize * self.height as usize {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn apply_vertical_tunnel(&mut self, y1:i32, y2:i32, x:i32) {
        for y in min(y1,y2) ..= max(y1,y2) {
            let idx = self.xy_idx(x, y);
            if idx > 0 && idx < self.width as usize * self.height as usize {
                self.tiles[idx as usize] = TileType::Floor;
            }
        }
    }

    fn is_exit_valid(&self, x:i32, y:i32) -> bool {
        if x < 1 || x > self.width-1 || y < 1 || y > self.height-1 { return false; }
        let idx = self.xy_idx(x, y);
        !self.blocked[idx]
    }

    pub fn populate_blocked(&mut self) {
        for (i,tile) in self.tiles.iter_mut().enumerate() {
            self.blocked[i] = *tile == TileType::Wall;
        }
    }

    pub fn clear_content_index(&mut self) {
        for content in self.tile_content.iter_mut() {
            content.clear();
        }
    }

    /// Makes a new map using the algorithm from http://rogueliketutorials.com/tutorials/tcod/part-3/
    /// This gives a handful of random rooms and corridors joining them together.
    pub fn new_map_rooms_and_corridors() -> Map {
        let mut rng = RandomNumberGenerator::new();

        let mut map = Map{
            tiles : vec![TileType::Wall; MAPCOUNT],
            rooms : Vec::new(),
            width : MAPWIDTH as i32,
            height: MAPHEIGHT as i32,
            revealed_tiles : vec![false; MAPCOUNT],
            visible_tiles : vec![false; MAPCOUNT],
            blocked : vec![false; MAPCOUNT],
            tile_content : vec![Vec::new(); MAPCOUNT],
            noise_seed : 0,
            frame_count : 0
        };

        const MAX_ROOMS : i32 = 30;
        const MIN_SIZE : i32 = 6;
        const MAX_SIZE : i32 = 10;

        for _i in 0..MAX_ROOMS {
            let w = rng.range(MIN_SIZE, MAX_SIZE);
            let h = rng.range(MIN_SIZE, MAX_SIZE);
            let x = rng.roll_dice(1, map.width - w - 1) - 1;
            let y = rng.roll_dice(1, map.height - h - 1) - 1;
            let new_room = Rect::new(x, y, w, h);
            let mut ok = true;
            for other_room in map.rooms.iter() {
                if new_room.intersect(other_room) { ok = false }
            }
            if ok {
                map.apply_room_to_map(&new_room);

                if !map.rooms.is_empty() {
                    let (new_x, new_y) = new_room.center();
                    let (prev_x, prev_y) = map.rooms[map.rooms.len()-1].center();
                    if rng.range(0,2) == 1 {
                        map.apply_horizontal_tunnel(prev_x, new_x, prev_y);
                        map.apply_vertical_tunnel(prev_y, new_y, new_x);
                    } else {
                        map.apply_vertical_tunnel(prev_y, new_y, prev_x);
                        map.apply_horizontal_tunnel(prev_x, new_x, new_y);
                    }
                }

                map.rooms.push(new_room);
            }
        }

        let first_room = map.rooms[0];
        let first_room_center = first_room.center();

        let cmp_room_dist = |a:&Rect, b:&Rect| -> Ordering {
            let a_center = a.center();
            let b_center = b.center();
            let a_distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(a_center.0, a_center.1), Point::new(first_room_center.0, first_room_center.1));
            let b_distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(b_center.0, b_center.1), Point::new(first_room_center.0, first_room_center.1));
            return a_distance.partial_cmp(&b_distance).unwrap_or(Ordering::Equal);
        };
        // let mut spawn_rooms = map.rooms.clone();
        map.rooms.sort_by(&cmp_room_dist);

        let stairs_up_position = first_room_center;
        let stairs_up_idx = map.xy_idx(stairs_up_position.0, stairs_up_position.1);
        map.tiles[stairs_up_idx] = TileType::StairsUp;

        let stairs_down_position = map.rooms[map.rooms.len()-1].center();
        let stairs_down_idx = map.xy_idx(stairs_down_position.0, stairs_down_position.1);
        map.tiles[stairs_down_idx] = TileType::StairsDown;

        map
    }
}

impl BaseMap for Map {
    fn is_opaque(&self, idx:usize) -> bool {
        self.tiles[idx] == TileType::Wall
    }


    fn get_pathing_distance(&self, idx1:usize, idx2:usize) -> f32 {
        let w = self.width as usize;
        let p1 = Point::new(idx1 % w, idx1 / w);
        let p2 = Point::new(idx2 % w, idx2 / w);
        rltk::DistanceAlg::Pythagoras.distance2d(p1, p2)
    }

    fn get_available_exits(&self, idx:usize) -> rltk::SmallVec<[(usize, f32); 10]> {
        let mut exits = rltk::SmallVec::new();
        let x = idx as i32 % self.width;
        let y = idx as i32 / self.width;
        let w = self.width as usize;

        // Cardinal directions
        if self.is_exit_valid(x-1, y) { exits.push((idx-1, 1.0)) };
        if self.is_exit_valid(x+1, y) { exits.push((idx+1, 1.0)) };
        if self.is_exit_valid(x, y-1) { exits.push((idx-w, 1.0)) };
        if self.is_exit_valid(x, y+1) { exits.push((idx+w, 1.0)) };

        // Diagonals
        if self.is_exit_valid(x-1, y-1) { exits.push(((idx-w)-1, 1.45)); }
        if self.is_exit_valid(x+1, y-1) { exits.push(((idx-w)+1, 1.45)); }
        if self.is_exit_valid(x-1, y+1) { exits.push(((idx+w)-1, 1.45)); }
        if self.is_exit_valid(x+1, y+1) { exits.push(((idx+w)+1, 1.45)); }

        exits
    }
}

impl Algorithm2D for Map {
    fn dimensions(&self) -> Point {
        Point::new(self.width, self.height)
    }
}

pub fn draw_map(ecs: &World, ctx : &mut Rltk) {
    let mut map = ecs.fetch_mut::<Map>();
    let mut noise = FastNoise::new();
    let player_ent = ecs.fetch::<Entity>();
    let positions = ecs.read_storage::<Position>();

    let player_ent_pos = positions.get(*player_ent).unwrap();
    let player_pos = Point::new(player_ent_pos.x,player_ent_pos.y);

    noise.set_seed(map.noise_seed);

    map.frame_count += 1;


    let mut y = 0;
    let mut x = 0;
    for (idx,tile) in map.tiles.iter().enumerate() {
        // Render a tile depending upon the tile type

        if map.revealed_tiles[idx] {
            let glyph;
            let mut fg;
            let mut bg;

            let distance = rltk::DistanceAlg::Pythagoras.distance2d(Point::new(x, y), player_pos);
            let dist_factor = 1.0 - ((distance - 3.0).max(0.0)/ 9.0) - (noise.get_noise3d(0.08 * x as f32, 0.08 * y as f32, 0.14 * map.frame_count as f32) * 0.1);
            // console::log(format!("{},{} distance: {}, dist_factor: {}", x,y, distance, dist_factor));
            

            match tile {
                TileType::Floor => {
                    glyph = rltk::to_cp437('.');
                    fg = RGB::from_f32(
                        0.25 * dist_factor, 
                        0.2 * dist_factor, 
                        0.15 * dist_factor
                    );
                    // fg = RGB::from_f32(0.25, 0.2, 0.15);

                    bg = RGB::from_f32(
                        dist_factor * 0.1,
                        dist_factor * 0.07,
                        dist_factor * 0.05);
                }
                TileType::Wall => {
                    glyph = rltk::to_cp437('#');
                    // fg = RGB::from_f32(
                    //     dist_factor * (0.4 + 0.04 * (noise.get_noise(1000.0 + x as f32, 800.0 + y as f32))), 
                    //     dist_factor * (0.2 + 0.08 * (noise.get_noise(500.0 + x as f32, 700.0 + y as f32))),
                    //     dist_factor * (0.0 + 0.05 * (noise.get_noise(300.0 + x as f32, 600.0 + y as f32)))
                    // );
                    fg = RGB::from_f32(
                        dist_factor * (0.5 + 0.08 * (noise.get_noise(500.0 + x as f32, 800.0 + y as f32)) ), 
                        dist_factor * (0.5 + 0.08 * (noise.get_noise(500.0 + x as f32, 800.0 + y as f32)) ),
                        dist_factor * (0.5 + 0.08 * (noise.get_noise(500.0 + x as f32, 800.0 + y as f32)) )
                    );

                    bg = RGB::from_f32(0.15,0.1,0.0);
                }
                TileType::StairsUp => {
                    glyph = rltk::to_cp437('<');
                    fg = RGB::from_f32(0.65,0.65,0.65);
                    bg = RGB::from_f32(0.15,0.1,0.0);
                }
                TileType::StairsDown => {
                    glyph = rltk::to_cp437('>');
                    fg = RGB::from_f32(0.65,0.65,0.65);
                    bg = RGB::from_f32(0.15,0.1,0.0);
                }
            }
            if !map.visible_tiles[idx] {
                 match tile { 
                    TileType::Floor => {
                        fg = RGB::named(rltk::BLACK);
                        bg = RGB::named(rltk::BLACK);
                    }
                    TileType::Wall => {
                        // fg = RGB::from_f32(
                        //     0.18 + 0.02 * (noise.get_noise(1000.0 + x as f32, 800.0 + y as f32)), 
                        //     0.1 + 0.03 * (noise.get_noise(500.0 + x as f32, 700.0 + y as f32)),
                        //     0.0 + 0.02 * (noise.get_noise(300.0 + x as f32, 600.0 + y as f32))
                        // );                        
                        fg = RGB::from_f32(
                            0.25 + 0.05 * (noise.get_noise(500.0 + x as f32, 800.0 + y as f32)), 
                            0.25 + 0.05 * (noise.get_noise(500.0 + x as f32, 800.0 + y as f32)),
                            0.25 + 0.05 * (noise.get_noise(500.0 + x as f32, 800.0 + y as f32))
                        );
                        bg = RGB::named(rltk::BLACK);
                    }
                    TileType::StairsUp => {
                        fg = RGB::from_f32(0.4,0.4,0.4);
                        bg = RGB::named(rltk::BLACK);
                    }
                    TileType::StairsDown => {
                        fg = RGB::from_f32(0.4,0.4,0.4);
                        bg = RGB::named(rltk::BLACK);
                    }
    
                }
            }
            ctx.set(x, y, fg, bg, glyph);
        }

        // Move the coordinates
        x += 1;
        if x > MAPWIDTH as i32-1 {
            x = 0;
            y += 1;
        }
    }
}
