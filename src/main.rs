use std::{any::Any, collections::HashMap, fmt::Display};

use assert_json_diff::assert_json_eq;
use serde::{Deserialize, Serialize};
use treediff::{diff, tools::Recorder};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct NodeCollection {
    nodes: HashMap<Uuid, Box<dyn GeometryNode>>,
}

fn concrete_node<'a, T>(node: &'a dyn GeometryNode) -> Option<&'a T>
where
    T: GeometryNode + 'static,
{
    node.as_any().downcast_ref::<T>()
}

fn concrete_node_mut<'a, T>(node: &'a mut dyn GeometryNode) -> Option<&'a mut T>
where
    T: GeometryNode + 'static,
{
    node.as_any_mut().downcast_mut::<T>()
}

impl NodeCollection {
    fn new() -> Self {
        Self {
            nodes: HashMap::new(),
        }
    }

    pub fn push(&mut self, node: Box<dyn GeometryNode>) {
        self.nodes.insert(node.uuid(), node);
    }

    pub fn remove(&mut self, key: &Uuid) -> Option<Box<dyn GeometryNode>> {
        self.nodes.remove(&key)
    }

    pub fn try_get_typed<'a, T>(&'a self, key: &'a Uuid) -> Option<&'a T>
    where
        T: GeometryNode + 'static,
    {
        self.nodes
            .get(key)
            .and_then(|n| concrete_node::<T>(n.as_ref()))
    }

    pub fn try_get_typed_mut<'a, T>(&'a mut self, key: &'a Uuid) -> Option<&'a mut T>
    where
        T: GeometryNode + 'static,
    {
        self.nodes
            .get_mut(key)
            .and_then(|n| concrete_node_mut::<T>(n.as_mut()))
    }
}

/// # TODO
/// A macro to implement uuid(), as_any() and as_any_mut()
/// for structs that implement GeometryNode.
/// 
/// Also think about if we could rather represent the whole architecture with an ecs.
/// Could give exactly the flexibility we want, while also being super granular with changes.
/// I think I actually like that more...
#[typetag::serde(tag = "geometry_node")]
trait GeometryNode {
    fn uuid(&self) -> Uuid;
    fn as_any(&self) -> &dyn Any;
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
struct Point3 {
    x: f64,
    y: f64,
    z: f64,
    uuid: Uuid,
}

impl Point3 {
    pub fn new() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            uuid: Uuid::new_v4(),
        }
    }
}

#[typetag::serde]
impl GeometryNode for Point3 {
    fn uuid(&self) -> Uuid {
        self.uuid
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Serialize, Deserialize)]
struct Rectangle {
    anchor: Point3,
    width: f64,
    height: f64,
    uuid: Uuid,
}

impl Rectangle {
    pub fn new() -> Self {
        Self {
            anchor: Point3::new(),
            width: 0.0,
            height: 0.0,
            uuid: Uuid::new_v4(),
        }
    }

    pub fn width_mut(&mut self) -> &mut f64 {
        &mut self.width
    }
    pub fn height_mut(&mut self) -> &mut f64 {
        &mut self.height
    }
}

#[typetag::serde]
impl GeometryNode for Rectangle {
    fn uuid(&self) -> Uuid {
        self.uuid
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

fn print_diff<'a, K, V>(recorder: &Recorder<'a, K, V>)
where
    V: Display,
{
    for call in &recorder.calls {
        match call {
            treediff::tools::ChangeType::Removed(_, v) => println!("removed {}", v),
            treediff::tools::ChangeType::Added(_, v) => println!("added {}", v),
            treediff::tools::ChangeType::Unchanged(_, v) => println!("entry unchanged {}", v),
            treediff::tools::ChangeType::Modified(_, old, new) => {
                println!("modified {} to {}", old, new)
            }
        };
    }
}

fn main() {
    println!("Hello, world!");

    let mut rect = Rectangle::new();
    *rect.width_mut() = 10.0;
    *rect.height_mut() = 20.0;
    let id = rect.uuid();

    let pt = Point3::new();
    let pt_id = pt.uuid();

    let mut nodes = NodeCollection::new();
    nodes.push(Box::new(rect));
    nodes.push(Box::new(pt));

    let naive = serde_json::to_value(&nodes).unwrap();

    if let Some(rect) = nodes.try_get_typed_mut::<Rectangle>(&id) {
        rect.anchor = pt;
    }

    let optimized = serde_json::to_value(&nodes).unwrap();

    let mut d = Recorder::default();
    diff(&naive, &optimized, &mut d);
    println!("Naive diff");
    print_diff(&d);

    let mut deser: NodeCollection = serde_json::from_value(optimized.clone()).unwrap();
    if let Some(pt) = deser.try_get_typed_mut::<Point3>(&pt_id) {
        pt.x = 50.0;
        pt.y = 100.0;
    };

    let mut d = Recorder::default();
    let dv = serde_json::to_value(deser).unwrap();
    diff(&optimized, &dv, &mut d);
    println!("Changed point diff");
    print_diff(&d);
}
