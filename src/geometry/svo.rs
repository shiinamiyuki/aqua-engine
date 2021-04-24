use core::panic;
use std::collections::HashMap;

use super::{Bound3, TriangleMesh};
use crate::glm;
pub struct SVO<T>
where
    T: Copy + Clone,
{
    pub bound: Bound3<f32>,
    pub max_depth: u32,
    pub nodes: Vec<SVONode<T>>,
}
#[derive(Clone, Copy)]
pub struct SVONode<T>
where
    T: Copy + Clone,
{
    // pub pos: u32, // encoded in u32,  [000..zyxzyxzyx]
    pub value: Option<T>,
    pub children: [Option<u32>; 8],
}

fn get_bit(x: u32, b: u32) -> u32 {
    (x & (1 << b)) >> b
}
pub fn uvec3_to_svo_index(v: &glm::UVec3) -> u32 {
    assert!(v[0] <= 10 && v[1] <= 10 && v[2] <= 10);
    let mut idx: u32 = 0;
    let mut shift: u32 = 0;
    for i in 0..10 {
        let cur = (
            get_bit(v[0], shift),
            get_bit(v[1], shift),
            get_bit(v[2], shift),
        );
        idx = ((cur.0 | (cur.1 << 1) | (cur.2 << 2)) << shift) | idx;
        shift += 1;
    }
    idx
}
impl<T> SVONode<T>
where
    T: Copy + Clone,
{
    pub fn is_leaf(&self) -> bool {
        self.children.iter().all(|x| x.is_none())
    }
}
impl<T> SVO<T>
where
    T: Copy + Clone,
{
    pub fn new(lo_corner: glm::Vec3, size: f32, max_depth: u32) -> Self {
        if max_depth > 10 {
            panic!("max_depth exceeds 10");
        }
        Self {
            bound: Bound3::<f32> {
                min: lo_corner,
                max: lo_corner + glm::vec3(size, size, size),
            },
            max_depth,
            nodes: vec![],
        }
    }
    pub fn voxel_size(&self) -> f32 {
        (self.bound.size() / (2u32.pow(self.max_depth) as f32))[0]
    }
    pub fn get(&self, p: &glm::Vec3) -> Option<T> {
        if self.nodes.is_empty() {
            return None;
        }
        let mut p_offset = self.bound.offset(&p);
        if glm::any(&glm::less_than(&p_offset, &glm::vec3(0.0, 0.0, 0.0)))
            || glm::any(&glm::greater_than(&p_offset, &glm::vec3(1.0, 1.0, 1.0)))
        {
            return None;
        }
        let mut ptr: Option<u32> = None;
        loop {
            let node = &self.nodes[ptr.unwrap() as usize];
            if node.is_leaf() {
                return Some(node.value.unwrap());
            }
            let mask = glm::less_than(&p_offset, &glm::vec3(0.5, 0.5, 0.5));
            let x = mask[0] as u32;
            let y = mask[1] as u32;
            let z = mask[2] as u32;
            let child_idx = x + 2 * y + 4 * z;
            ptr = Some(node.children[child_idx as usize].unwrap());
            for i in 0..3 {
                if mask[i] {
                    p_offset[i] *= 2.0;
                } else {
                    p_offset[i] = (p_offset[i] - 0.5) * 2.0;
                }
            }
        }
    }
    pub fn put(&mut self, p: &glm::Vec3, val: T) {
        if self.nodes.is_empty() {
            self.nodes.push(SVONode::<T> {
                value: Some(val),
                children: [None; 8],
            });
        } else {
            let mut p_offset = self.bound.offset(&p);
            if glm::any(&glm::less_than(&p_offset, &glm::vec3(0.0, 0.0, 0.0)))
                || glm::any(&glm::greater_than(&p_offset, &glm::vec3(1.0, 1.0, 1.0)))
            {
                return;
            }
            let mut ptr: Option<u32> = None;
            let mut depth = 0;
            while depth < self.max_depth {
                let mut node = self.nodes[ptr.unwrap() as usize];
                if node.is_leaf() {
                    let mut children = [None; 8];
                    for i in 0..8 {
                        self.nodes.push(SVONode::<T> {
                            value: None,
                            children: [None; 8],
                        });
                        children[i] = Some(self.nodes.len() as u32);
                    }
                    node.children = children;
                    self.nodes[ptr.unwrap() as usize] = node;
                }
                let mask = glm::less_than(&p_offset, &glm::vec3(0.5, 0.5, 0.5));
                let x = mask[0] as u32;
                let y = mask[1] as u32;
                let z = mask[2] as u32;
                let child_idx = x + 2 * y + 4 * z;
                ptr = Some(node.children[child_idx as usize].unwrap());
                for i in 0..3 {
                    if mask[i] {
                        p_offset[i] *= 2.0;
                    } else {
                        p_offset[i] = (p_offset[i] - 0.5) * 2.0;
                    }
                }
                depth += 1;
            }
            let node = &mut self.nodes[ptr.unwrap() as usize];
            node.value = Some(val);
        }
    }
    fn compress_node(&self, idx: u32, new_nodes: &mut Vec<SVONode<T>>) -> u32 {
        let node = &self.nodes[idx as usize];
        if node.is_leaf() {
            new_nodes.push(*node);
            let ret = new_nodes.len() as u32;
            return ret;
        }
        let children: Vec<u32> = node
            .children
            .iter()
            .filter(|x| x.is_some())
            .map(|x| x.unwrap())
            .collect();
        if children.len() == 0 {
            unreachable!();
        }
        if children.len() == 1 {
            return self.compress_node(children[0], new_nodes);
        }
        let mut new_node = SVONode::<T> {
            value: None,
            children: [None; 8],
        };
        new_nodes.push(new_node);
        let ret = new_nodes.len() as u32;
        for i in 0..8 {
            if node.children[i].is_some() {
                new_node.children[i] =
                    Some(self.compress_node(node.children[i].unwrap(), new_nodes));
            }
        }
        new_nodes[ret as usize] = new_node;
        ret
    }
    pub fn compress(&self) -> Self {
        let mut new_nodes = Vec::new();
        let _ = self.compress_node(0, &mut new_nodes);
        Self {
            bound: self.bound,
            max_depth: self.max_depth,
            nodes: new_nodes,
        }
    }
}

pub type SVOf32x4 = SVO<[f32; 4]>;

pub fn build_svo_from_mesh(svo: &mut SVOf32x4, mesh: &TriangleMesh) {
    let voxel_size = svo.voxel_size();
    for face in mesh.indices.iter() {
        let triangle = [
            mesh.vertices[face[0] as usize],
            mesh.vertices[face[1] as usize],
            mesh.vertices[face[2] as usize],
        ];
        let sorted = {
            let mut tmp = triangle;
            tmp.sort_by(|a, b| a.y.partial_cmp(&b.y).unwrap());
            tmp
        };
        let top = sorted[0];
        let bot = sorted[2];
        let mid = sorted[1];

        if top.y > mid.y && top.y > bot.y {
            let mut y: f32 = top.y;
            while y > mid.y {
                let t1 = (y - mid.y) / (top.y - mid.y);
                let t2 = (y - bot.y) / (top.y - bot.y);
                let p1 = glm::lerp(&mid, &top, t1);
                let p2 = glm::lerp(&bot, &top, t2);
                let lp1p2 = glm::length(&(p1 - p2));
                let mut t0 = 0.0;
                let mut flag = false;
                loop {
                    let p = glm::lerp(&p1, &p2, t0 / lp1p2);
                    svo.put(&p, [1.0, 0.0, 0.0, 0.0]);
                    t0 += voxel_size * 0.99;
                    if t0 > lp1p2 {
                        if flag {
                            t0 = lp1p2;
                            flag = false;
                        } else {
                            break;
                        }
                    }
                }
                y -= voxel_size * 0.99;
            }
        }
        if mid.y > bot.y && top.y > bot.y {
            let mut y: f32 = mid.y;
            let mut flag0 = true;
            loop {
                let t1 = (y - mid.y) / (mid.y - bot.y);
                let t2 = (y - bot.y) / (top.y - bot.y);
                let p1 = glm::lerp(&bot, &mid, t1);
                let p2 = glm::lerp(&bot, &top, t2);
                let lp1p2 = glm::length(&(p1 - p2));
                let mut t0 = 0.0;
                let mut flag = true;
                loop {
                    let p = glm::lerp(&p1, &p2, t0 / lp1p2);
                    svo.put(&p, [1.0, 0.0, 0.0, 0.0]);
                    t0 += voxel_size * 0.99;
                    if t0 > lp1p2 {
                        if flag {
                            t0 = lp1p2;
                            flag = false;
                        } else {
                            break;
                        }
                    }
                }
                y -= voxel_size * 0.99;
                if y >= bot.y {
                    if flag0 {
                        flag0 = false;
                        y = bot.y;
                    } else {
                        break;
                    }
                }
            }
        }
    }
}
