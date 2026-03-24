//! 场景树 - 管理所有场景节点的层级关系

use crate::SceneNodeData;
use indexmap::IndexMap;
use std::collections::HashMap;
use ummerse_core::error::{EngineError, Result};
use ummerse_core::node::{NodeId, NodeType};

/// 场景树 - 类似 Godot 的 SceneTree
impl std::fmt::Debug for SceneTree {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SceneTree")
            .field("node_count", &self.nodes.len())
            .field("root", &self.root)
            .finish_non_exhaustive()
    }
}

pub struct SceneTree {
    /// 所有节点（按 ID 索引）
    nodes: HashMap<NodeId, SceneNodeData>,
    /// 根节点 ID
    root: Option<NodeId>,
    /// 名称到 ID 的快速查找（路径 -> ID）
    path_cache: IndexMap<String, NodeId>,
}

impl SceneTree {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            root: None,
            path_cache: IndexMap::new(),
        }
    }

    /// 创建并设置根节点
    pub fn create_root(&mut self, name: impl Into<String>) -> NodeId {
        let name = name.into();
        let data = SceneNodeData::new(name.clone(), NodeType::Node);
        let id = data.id;
        self.root = Some(id);
        self.path_cache.insert(format!("/{}", name), id);
        self.nodes.insert(id, data);
        id
    }

    /// 添加节点到父节点下
    pub fn add_node(
        &mut self,
        mut node: SceneNodeData,
        parent_id: Option<NodeId>,
    ) -> Result<NodeId> {
        let id = node.id;

        // 设置父节点
        node.parent = parent_id;

        // 更新父节点的 children 列表
        if let Some(pid) = parent_id {
            let parent = self
                .nodes
                .get_mut(&pid)
                .ok_or_else(|| EngineError::NodeNotFound {
                    path: pid.to_string(),
                })?;
            parent.children.push(id);
        } else if self.root.is_none() {
            self.root = Some(id);
        }

        // 计算路径并缓存
        let path = self.compute_path(parent_id, &node.name);
        self.path_cache.insert(path, id);
        self.nodes.insert(id, node);
        Ok(id)
    }

    /// 根据 ID 获取节点
    pub fn get(&self, id: NodeId) -> Option<&SceneNodeData> {
        self.nodes.get(&id)
    }

    /// 根据 ID 获取可变节点
    pub fn get_mut(&mut self, id: NodeId) -> Option<&mut SceneNodeData> {
        self.nodes.get_mut(&id)
    }

    /// 根据路径获取节点
    pub fn get_by_path(&self, path: &str) -> Option<&SceneNodeData> {
        let id = self.path_cache.get(path)?;
        self.nodes.get(id)
    }

    /// 根据名称查找节点（深度优先搜索）
    pub fn find_by_name(&self, name: &str) -> Option<&SceneNodeData> {
        self.nodes.values().find(|n| n.name == name)
    }

    /// 获取节点的子节点列表
    pub fn children_of(&self, id: NodeId) -> Vec<&SceneNodeData> {
        if let Some(node) = self.nodes.get(&id) {
            node.children
                .iter()
                .filter_map(|&cid| self.nodes.get(&cid))
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 获取节点路径
    pub fn path_of(&self, id: NodeId) -> Option<String> {
        self.path_cache
            .iter()
            .find(|&(_, v)| *v == id)
            .map(|(k, _)| k.clone())
    }

    /// 移除节点（递归移除所有子节点）
    pub fn remove_node(&mut self, id: NodeId) -> Result<()> {
        // 收集所有要移除的节点 ID（深度优先）
        let mut to_remove = Vec::new();
        self.collect_subtree(id, &mut to_remove);

        // 从父节点移除引用
        if let Some(node) = self.nodes.get(&id)
            && let Some(pid) = node.parent
            && let Some(parent) = self.nodes.get_mut(&pid)
        {
            parent.children.retain(|&c| c != id);
        }

        // 移除所有节点
        for nid in to_remove {
            self.nodes.remove(&nid);
            // 清理路径缓存
            self.path_cache.retain(|_, v| *v != nid);
        }
        Ok(())
    }

    /// 重命名节点
    pub fn rename_node(&mut self, id: NodeId, new_name: impl Into<String>) -> Result<()> {
        let new_name = new_name.into();

        // 获取旧路径
        let old_path = self
            .path_cache
            .iter()
            .find(|&(_, v)| *v == id)
            .map(|(k, _)| k.clone());

        if let Some(node) = self.nodes.get_mut(&id) {
            node.name = new_name.clone();
        } else {
            return Err(EngineError::NodeNotFound {
                path: id.to_string(),
            });
        }

        // 更新路径缓存
        if let Some(old_path) = old_path {
            self.path_cache.shift_remove(&old_path);
            let parent_id = self.nodes.get(&id).and_then(|n| n.parent);
            let new_path = self.compute_path(parent_id, &new_name);
            self.path_cache.insert(new_path, id);
        }

        Ok(())
    }

    /// 移动节点到新父节点下
    pub fn reparent(&mut self, id: NodeId, new_parent: Option<NodeId>) -> Result<()> {
        // 从旧父节点移除
        let old_parent = self.nodes.get(&id).and_then(|n| n.parent);
        if let Some(old_pid) = old_parent
            && let Some(old_parent_node) = self.nodes.get_mut(&old_pid)
        {
            old_parent_node.children.retain(|&c| c != id);
        }

        // 添加到新父节点
        if let Some(new_pid) = new_parent {
            let parent = self
                .nodes
                .get_mut(&new_pid)
                .ok_or_else(|| EngineError::NodeNotFound {
                    path: new_pid.to_string(),
                })?;
            parent.children.push(id);
        }

        // 更新节点 parent 字段
        if let Some(node) = self.nodes.get_mut(&id) {
            node.parent = new_parent;
        }

        // 重建路径缓存（简化：完整实现需要递归更新子树）
        self.rebuild_path_cache();

        Ok(())
    }

    /// 重建路径缓存（完整重建）
    fn rebuild_path_cache(&mut self) {
        self.path_cache.clear();
        if let Some(root_id) = self.root {
            self.rebuild_path_recursive(root_id, "/");
        }
    }

    fn rebuild_path_recursive(&mut self, id: NodeId, parent_path: &str) {
        let (name, children) = if let Some(node) = self.nodes.get(&id) {
            (node.name.clone(), node.children.clone())
        } else {
            return;
        };

        let path = if parent_path == "/" {
            format!("/{}", name)
        } else {
            format!("{}/{}", parent_path, name)
        };

        self.path_cache.insert(path.clone(), id);
        for child_id in children {
            self.rebuild_path_recursive(child_id, &path);
        }
    }

    /// 递归收集子树所有节点 ID
    fn collect_subtree(&self, id: NodeId, result: &mut Vec<NodeId>) {
        result.push(id);
        if let Some(node) = self.nodes.get(&id) {
            for &child_id in &node.children {
                self.collect_subtree(child_id, result);
            }
        }
    }

    /// 计算节点路径
    fn compute_path(&self, parent_id: Option<NodeId>, name: &str) -> String {
        if let Some(pid) = parent_id {
            // 查找父路径
            if let Some((parent_path, _)) = self.path_cache.iter().find(|&(_, v)| *v == pid) {
                return format!("{}/{}", parent_path, name);
            }
        }
        format!("/{}", name)
    }

    /// 获取根节点 ID
    pub fn root(&self) -> Option<NodeId> {
        self.root
    }

    /// 节点总数
    pub fn node_count(&self) -> usize {
        self.nodes.len()
    }

    /// 遍历所有节点（广度优先），返回有序 ID 列表
    pub fn bfs_order(&self) -> Vec<NodeId> {
        let mut order = Vec::new();
        if let Some(root_id) = self.root {
            let mut queue = std::collections::VecDeque::new();
            queue.push_back(root_id);
            while let Some(id) = queue.pop_front() {
                if let Some(node) = self.nodes.get(&id) {
                    order.push(id);
                    for &child_id in &node.children {
                        queue.push_back(child_id);
                    }
                }
            }
        }
        order
    }

    /// 遍历所有节点（广度优先）
    pub fn iter_bfs(&self) -> impl Iterator<Item = &SceneNodeData> {
        let order = self.bfs_order();
        order
            .into_iter()
            .filter_map(move |id| self.nodes.get(&id))
            .collect::<Vec<_>>()
            .into_iter()
    }

    /// 获取节点深度（根节点为 0）
    pub fn depth_of(&self, id: NodeId) -> usize {
        let mut depth = 0;
        let mut current = id;
        while let Some(node) = self.nodes.get(&current) {
            if let Some(parent_id) = node.parent {
                depth += 1;
                current = parent_id;
            } else {
                break;
            }
        }
        depth
    }

    /// 判断节点是否是另一个节点的祖先
    pub fn is_ancestor_of(&self, ancestor: NodeId, descendant: NodeId) -> bool {
        let mut current = descendant;
        loop {
            if let Some(node) = self.nodes.get(&current) {
                if let Some(parent_id) = node.parent {
                    if parent_id == ancestor {
                        return true;
                    }
                    current = parent_id;
                } else {
                    return false;
                }
            } else {
                return false;
            }
        }
    }

    /// 按标签查找所有节点
    pub fn find_by_tag(&self, tag: &str) -> Vec<&SceneNodeData> {
        self.nodes.values().filter(|n| n.has_tag(tag)).collect()
    }

    /// 按节点类型查找所有节点
    pub fn find_by_type(&self, node_type: &NodeType) -> Vec<&SceneNodeData> {
        self.nodes
            .values()
            .filter(|n| &n.node_type == node_type)
            .collect()
    }

    /// 获取节点的所有祖先（从直接父节点到根节点）
    pub fn ancestors_of(&self, id: NodeId) -> Vec<&SceneNodeData> {
        let mut result = Vec::new();
        let mut current = id;
        while let Some(node) = self.nodes.get(&current) {
            if let Some(parent_id) = node.parent {
                if let Some(parent) = self.nodes.get(&parent_id) {
                    result.push(parent);
                    current = parent_id;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
        result
    }

    /// 获取节点的兄弟节点
    pub fn siblings_of(&self, id: NodeId) -> Vec<&SceneNodeData> {
        let parent_id = self.nodes.get(&id).and_then(|n| n.parent);
        if let Some(pid) = parent_id {
            self.children_of(pid)
                .into_iter()
                .filter(|n| n.id != id)
                .collect()
        } else {
            Vec::new()
        }
    }

    /// 启用/禁用节点
    pub fn set_enabled(&mut self, id: NodeId, enabled: bool) -> bool {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.enabled = enabled;
            true
        } else {
            false
        }
    }

    /// 显示/隐藏节点
    pub fn set_visible(&mut self, id: NodeId, visible: bool) -> bool {
        if let Some(node) = self.nodes.get_mut(&id) {
            node.visible = visible;
            true
        } else {
            false
        }
    }

    /// 复制节点（深拷贝，含子树，赋予新 ID）
    pub fn duplicate(&mut self, id: NodeId, parent_id: Option<NodeId>) -> Result<NodeId> {
        // 收集子树快照
        let mut subtree: Vec<SceneNodeData> = Vec::new();
        self.collect_subtree_data(id, &mut subtree);

        if subtree.is_empty() {
            return Err(EngineError::NodeNotFound {
                path: id.to_string(),
            });
        }

        // 构建旧 ID → 新 ID 映射
        use std::collections::HashMap;
        let mut id_map: HashMap<NodeId, NodeId> = HashMap::new();
        for node in &subtree {
            id_map.insert(node.id, NodeId::new());
        }

        // 重建节点，替换 ID
        let mut new_root_id: Option<NodeId> = None;
        for (idx, node) in subtree.into_iter().enumerate() {
            let new_id = *id_map.get(&node.id).unwrap();
            let new_parent = if idx == 0 {
                parent_id
            } else {
                node.parent.and_then(|pid| id_map.get(&pid)).copied()
            };
            let mut new_node = node.clone();
            new_node.id = new_id;
            new_node.parent = new_parent;
            new_node.children = node
                .children
                .iter()
                .filter_map(|cid| id_map.get(cid))
                .copied()
                .collect();

            if idx == 0 {
                new_root_id = Some(new_id);
            }

            // 添加到父节点的 children
            if let Some(pid) = new_parent
                && let Some(parent_node) = self.nodes.get_mut(&pid)
            {
                parent_node.children.push(new_id);
            }
            self.nodes.insert(new_id, new_node);
        }

        self.rebuild_path_cache();
        new_root_id.ok_or_else(|| EngineError::NodeNotFound {
            path: id.to_string(),
        })
    }

    /// 收集子树快照（用于 duplicate）
    fn collect_subtree_data(&self, id: NodeId, result: &mut Vec<SceneNodeData>) {
        if let Some(node) = self.nodes.get(&id) {
            result.push(node.clone());
            for &child_id in &node.children.clone() {
                self.collect_subtree_data(child_id, result);
            }
        }
    }

    /// 将场景树扁平化为 Vec<SceneNodeData>（用于序列化）
    pub fn flatten(&self) -> Vec<SceneNodeData> {
        self.bfs_order()
            .into_iter()
            .filter_map(|id| self.nodes.get(&id).cloned())
            .collect()
    }

    /// 从扁平列表恢复场景树（用于反序列化）
    pub fn from_flat(nodes: Vec<SceneNodeData>, root_id: Option<NodeId>) -> Self {
        let mut tree = Self::new();
        tree.root = root_id;
        for node in nodes {
            tree.nodes.insert(node.id, node);
        }
        tree.rebuild_path_cache();
        tree
    }
}

impl Default for SceneTree {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_root() {
        let mut tree = SceneTree::new();
        let root_id = tree.create_root("Root");
        assert!(tree.root().is_some());
        assert_eq!(tree.node_count(), 1);
        assert!(tree.get(root_id).is_some());
    }

    #[test]
    fn test_add_child_node() {
        let mut tree = SceneTree::new();
        let root_id = tree.create_root("Root");
        let child = SceneNodeData::new("Player", NodeType::Node2d);
        let child_id = tree.add_node(child, Some(root_id)).unwrap();
        assert_eq!(tree.node_count(), 2);
        let root = tree.get(root_id).unwrap();
        assert!(root.children.contains(&child_id));
    }

    #[test]
    fn test_path_lookup() {
        let mut tree = SceneTree::new();
        let root_id = tree.create_root("Root");
        let child = SceneNodeData::new("Player", NodeType::Node2d);
        tree.add_node(child, Some(root_id)).unwrap();
        assert!(tree.get_by_path("/Root/Player").is_some());
    }

    #[test]
    fn test_remove_node() {
        let mut tree = SceneTree::new();
        let root_id = tree.create_root("Root");
        let child = SceneNodeData::new("Enemy", NodeType::Node2d);
        let child_id = tree.add_node(child, Some(root_id)).unwrap();
        tree.remove_node(child_id).unwrap();
        assert_eq!(tree.node_count(), 1);
    }
}
