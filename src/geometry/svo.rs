// use core::panic;
// use std::collections::HashMap;

// use super::Bound3;
// use crate::glm;
// pub struct SVO<T>
// where
//     T: Copy + Clone,
// {
//     pub bound: Bound3<f32>,
//     pub max_depth: u32,
//     pub nodes: Vec<SVONode<T>>,
// }
// #[derive(Clone, Copy)]
// pub struct SVONode<T>
// where
//     T: Copy + Clone,
// {
//     // pub pos: u32, // encoded in u32,  [000..zyxzyxzyx]
//     pub value: Option<T>,
//     pub children: [Option<u32>;8],
// }

// fn get_bit(x: u32, b: u32) -> u32 {
//     (x & (1 << b)) >> b
// }
// pub fn uvec3_to_svo_index(v: &glm::UVec3) -> u32 {
//     assert!(v[0] <= 10 && v[1] <= 10 && v[2] <= 10);
//     let mut idx: u32 = 0;
//     let mut shift: u32 = 0;
//     for i in 0..10 {
//         let cur = (
//             get_bit(v[0], shift),
//             get_bit(v[1], shift),
//             get_bit(v[2], shift),
//         );
//         idx = ((cur.0 | (cur.1 << 1) | (cur.2 << 2)) << shift) | idx;
//         shift += 1;
//     }
//     idx
// }
// impl<T> SVONode<T>
// where
//     T: Copy + Clone,
// {
//     pub fn is_leaf(&self) -> bool {
//         self.children.iter().all(|x|x.is_none())
//     }
// }
// impl<T> SVO<T>
// where
//     T: Copy + Clone,
// {
//     pub fn new(bound: &Bound3<f32>, max_depth: u32) -> Self {
//         if max_depth > 10 {
//             panic!("max_depth exceeds 10");
//         }
//         Self {
//             bound: *bound,
//             max_depth,
//             nodes: vec![],
//         }
//     }
//     pub fn get(&self, p: &glm::Vec3) -> Option<T> {
//         if self.nodes.is_empty() {
//             return None;
//         }
//         let mut p_offset = self.bound.offset(&p);
//         let mut ptr: Option<u32> = None;
//         while true {
//             let node = &self.nodes[ptr.unwrap() as usize];
//             if node.is_leaf() {
//                 return Some(node.value.unwrap());
//             }
//             let mask = glm::less_than(&p_offset, &glm::vec3(0.5, 0.5, 0.5));
//             let x = mask[0] as u32;
//             let y = mask[1] as u32;
//             let z = mask[2] as u32;
//             let child_idx = x + 2 * y + 4 * z;
//             ptr = Some(node.children[child_idx as usize].unwrap());
//             for i in 0..3 {
//                 if mask[i] {
//                     p_offset[i] *= 2.0;
//                 } else {
//                     p_offset[i] = (p_offset[i] - 0.5) * 2.0;
//                 }
//             }
//         }
//         None
//     }
//     pub fn put(&mut self, p: &glm::Vec3, val: T) {
//         if self.nodes.is_empty() {
//             self.nodes.push(SVONode::<T> {
//                 value: Some(val),
//                 children: [None;8],
//             });
//         } else {
//             let mut p_offset = self.bound.offset(&p);
//             let mut ptr: Option<u32> = None;
//             let mut depth = 0;
//             while depth < self.max_depth {
//                 let mut node = self.nodes[ptr.unwrap() as usize];
//                 if node.is_leaf() {
//                     let mut children=[None;8];
//                     for i in 0..8 {
//                         self.nodes.push(SVONode::<T> {
//                             value: None,
//                             children: [None;8],
//                         });
//                         children[i] = Some(self.nodes.len() as u32);
//                     }
//                     node.children = children;
//                     self.nodes[ptr.unwrap() as usize] = node;
//                 }
//                 let mask = glm::less_than(&p_offset, &glm::vec3(0.5, 0.5, 0.5));
//                 let x = mask[0] as u32;
//                 let y = mask[1] as u32;
//                 let z = mask[2] as u32;
//                 let child_idx = x + 2 * y + 4 * z;
//                 ptr = Some(node.children[child_idx as usize].unwrap());
//                 for i in 0..3 {
//                     if mask[i] {
//                         p_offset[i] *= 2.0;
//                     } else {
//                         p_offset[i] = (p_offset[i] - 0.5) * 2.0;
//                     }
//                 }
//                 depth += 1;
//             }
//             let node = &mut self.nodes[ptr.unwrap() as usize];
//             node.value = Some(val);
//         }
//     }

//     pub fn compress(&self) -> Self {
//         let dict = HashMap::new();
//         let new_nodes = Vec::new();
//         let compress_node = |idx: u32| -> u32 {
//             let node = &self.nodes[idx as usize];
//             if node.is_leaf() {
//                 new_nodes.push(*node);
//                 return new_nodes.len() as u32;
//             }
//             let children:Vec<u32> = node.children.iter().filter(|x|x.is_some()).map(|x|x.unwrap()).collect();
//             // if children.len() == 1 {
//             //     return compress_node()
//             // }
//             0
//         };
//         Self {
//             bound: self.bound,
//             max_depth: self.max_depth,
//             nodes: new_nodes,
//         }
//     }
// }
