use super::*;

pub type RankScore = f64;

/// Rank nodes using an adaptation of the page rank algorithm (no dampening, fixed number of runs,
/// no distinction between validators and inner quorum set validators). Links from nodes not in
/// `nodes` are ignored.
pub fn rank_nodes(nodes: &[NodeId], fbas: &Fbas) -> Vec<RankScore> {
    let nodes_set: NodeIdSet = nodes.iter().cloned().collect();
    assert_eq!(nodes.len(), nodes_set.len());

    let runs = 100;
    let starting_score = 1. / nodes.len() as RankScore;

    let mut scores: Vec<RankScore> = vec![starting_score; fbas.nodes.len()];
    let mut last_scores: Vec<RankScore>;

    for _ in 0..runs {
        last_scores = scores;
        scores = vec![0.; fbas.nodes.len()];

        for node_id in nodes.iter().copied() {
            let node = &fbas.nodes[node_id];
            let trusted_nodes = node.quorum_set.contained_nodes();
            let l = trusted_nodes.len() as RankScore;

            for node_id in trusted_nodes
                .into_iter()
                .filter(|&id| nodes_set.contains(id))
            {
                scores[node_id] += last_scores[node_id] / l;
            }
        }
    }
    scores
}

/// Rank nodes and sort them by "highest rank score first"
pub fn sort_by_rank(mut nodes: Vec<NodeId>, fbas: &Fbas) -> Vec<NodeId> {
    let scores = rank_nodes(&nodes, fbas);

    nodes.sort_by(|x, y| scores[*y].partial_cmp(&scores[*x]).unwrap());
    nodes
}