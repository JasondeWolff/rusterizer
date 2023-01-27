use std::collections::HashMap;

use crate::{Shared, timer::Timer};

pub struct ResourceManager<T: Clone> {
    resources: Vec<(Shared<T>, String)>,
    asset_paths: HashMap<String, Shared<T>>,
    kill_times: HashMap<*const T, f32>,

    timer: Timer,
    pub kill_time: f32
}

fn vec_remove_multiple<T>(vec: &mut Vec<T>, indices: &mut Vec<usize>) {
    indices.sort();    

    let mut j: usize = 0;
    for i in indices.iter() {
        vec.remove(i - j);
        j += 1;
    }
}

impl<T: Clone> ResourceManager<T> {
    pub fn new(kill_time: f32) -> Self {
        ResourceManager {
            resources: Vec::new(),
            asset_paths: HashMap::new(),
            kill_times: HashMap::new(),
            timer: Timer::new(),
            kill_time: kill_time,
        }
    }

    pub fn update(&mut self) {
        let mut resources_to_remove: Vec<(usize, String)> = Vec::new();

        for i in 0..self.resources.len() {
            let resource_ptr = self.resources[i].0.as_ptr();

            let use_count = self.resources[i].0.strong_count();
            if use_count <= 2 {
                match self.kill_times.get(&resource_ptr) {
                    Some(start_time) => {
                        if start_time + self.kill_time < self.timer.elapsed() as f32 {
                            resources_to_remove.push((i, self.resources[i].1.clone()));
                            self.kill_times.remove(&resource_ptr);
                        }
                    },
                    None => {
                        self.kill_times.insert(resource_ptr, self.timer.elapsed() as f32);
                    }
                }
            }
            else {
                self.kill_times.remove(&resource_ptr);
            }
        }

        if resources_to_remove.len() > 0 {
            let mut indices: Vec<usize> = Vec::with_capacity(resources_to_remove.len());
            let mut paths: Vec<String> = Vec::with_capacity(resources_to_remove.len());
            for i in resources_to_remove {
                indices.push(i.0);
                paths.push(i.1);
            }

            vec_remove_multiple(&mut self.resources, &mut indices);
            for path in paths {
                self.asset_paths.remove(&path);
            }
        }
    }

    pub fn get(&self, asset_path: &String) -> Option<Shared<T>> {
        self.asset_paths.get(asset_path).cloned()
    }

    pub fn insert(&mut self, resource: Shared<T>, asset_path: String) {
        self.resources.push((resource.clone(), asset_path.clone()));
        self.asset_paths.insert(asset_path, resource);
    }
}