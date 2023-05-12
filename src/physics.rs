use cgmath::{point2, Point2};

//works to check gravity
pub fn check_player_gravity_collission(
    player_pos: Point2<f32>,
    block_pos: Point2<f32>,
    quad_size: f32,
    block_length: Point2<usize>,
) -> Option<Point2<f32>> {
    //println!("Player pos {:?}, block pos {:?}", player_pos, block_pos);

    if player_pos.x + quad_size >= block_pos.x 
    && player_pos.x < block_pos.x + (quad_size * block_length.x as f32)
    && player_pos.y + quad_size > block_pos.y
    && player_pos.y <= block_pos.y + (quad_size * block_length.y as f32){
       //println!("colliding");
        return Some(point2::<f32>(player_pos.x, (block_pos.y + (quad_size * block_length.y as f32))));
    }
    None
}
