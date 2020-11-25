use super::*;

mod front_end;
mod results;

mod blocking_sets;
mod quorums;
mod splitting_sets;
mod symmetric_clusters;

mod merge_by_group;
pub mod preprocessing;
pub mod sets;
pub mod timing;

pub use front_end::Analysis;
pub use results::{NodeIdSetResult, NodeIdSetVecResult};

pub use blocking_sets::find_minimal_blocking_sets;
pub use quorums::{find_minimal_quorums, find_nonintersecting_quorums};
pub use splitting_sets::find_minimal_splitting_sets;
pub use symmetric_clusters::find_symmetric_clusters;

pub(crate) use preprocessing::*;
pub(crate) use quorums::*;
pub(crate) use sets::*;
pub(crate) use symmetric_clusters::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn has_quorum_intersection_trivial() {
        let correct = Fbas::from_json_file(Path::new("test_data/correct_trivial.json"));
        let broken = Fbas::from_json_file(Path::new("test_data/broken_trivial.json"));

        assert!(Analysis::new(&correct).has_quorum_intersection());
        assert!(!Analysis::new(&broken).has_quorum_intersection());
    }

    #[test]
    fn has_quorum_intersection_nontrivial() {
        let correct = Fbas::from_json_file(Path::new("test_data/correct.json"));
        let broken = Fbas::from_json_file(Path::new("test_data/broken.json"));

        assert!(Analysis::new(&correct).has_quorum_intersection());
        assert!(!Analysis::new(&broken).has_quorum_intersection());
    }

    #[test]
    fn has_quorum_intersection_if_just_one_quorum() {
        let fbas = Fbas::from_json_str(
            r#"[
            {
                "publicKey": "n0",
                "quorumSet": { "threshold": 2, "validators": ["n0", "n1"] }
            },
            {
                "publicKey": "n1",
                "quorumSet": { "threshold": 2, "validators": ["n0", "n1"] }
            }
        ]"#,
        );
        assert!(Analysis::new(&fbas).has_quorum_intersection());
    }

    #[test]
    fn no_has_quorum_intersection_if_there_is_no_quorum() {
        let fbas = Fbas::from_json_str(
            r#"[
            {
                "publicKey": "n0",
                "quorumSet": { "threshold": 2, "validators": ["n0", "n1"] }
            }
        ]"#,
        );
        assert!(!Analysis::new(&fbas).has_quorum_intersection());
    }

    #[test]
    fn analysis_nontrivial() {
        let fbas = Fbas::from_json_file(Path::new("test_data/correct.json"));
        let analysis = Analysis::new(&fbas);

        assert!(analysis.has_quorum_intersection());
        assert_eq!(
            analysis.minimal_quorums().describe(),
            NodeIdSetVecResult::new(vec![bitset![0, 1], bitset![0, 10], bitset![1, 10]], None)
                .describe()
        );
        assert_eq!(
            analysis.minimal_blocking_sets().describe(),
            NodeIdSetVecResult::new(vec![bitset![0, 1], bitset![0, 10], bitset![1, 10]], None)
                .describe()
        );
        assert_eq!(
            analysis.minimal_splitting_sets().describe(),
            NodeIdSetVecResult::new(vec![bitset![0], bitset![1], bitset![10]], None).describe()
        );
    }

    #[test]
    fn analysis_nontrivial_blocking_sets_first() {
        let fbas = Fbas::from_json_file(Path::new("test_data/correct.json"));
        let analysis = Analysis::new(&fbas);

        assert_eq!(
            analysis.minimal_blocking_sets().describe(),
            NodeIdSetVecResult::new(vec![bitset![0, 1], bitset![0, 10], bitset![1, 10]], None)
                .describe()
        );

        assert!(analysis.has_quorum_intersection());
        assert_eq!(
            analysis.minimal_quorums().describe(),
            NodeIdSetVecResult::new(vec![bitset![0, 1], bitset![0, 10], bitset![1, 10]], None)
                .describe()
        );
        assert_eq!(
            analysis.minimal_splitting_sets().describe(),
            NodeIdSetVecResult::new(vec![bitset![0], bitset![1], bitset![10]], None).describe()
        );
    }

    #[test]
    fn splitting_sets_on_broken() {
        let fbas = Fbas::from_json_file(Path::new("test_data/broken.json"));
        let analysis = Analysis::new(&fbas);

        let actual = analysis.minimal_splitting_sets().unwrap();
        let expected = bitsetvec![{}];

        assert_eq!(expected, actual);
    }

    #[test]
    fn alternative_check_on_broken() {
        let fbas = Fbas::from_json_file(Path::new("test_data/broken.json"));
        let analysis = Analysis::new(&fbas);

        let (has_intersection, quorums) = analysis.has_quorum_intersection_via_alternative_check();

        assert!(!has_intersection);

        let quorums: Vec<NodeIdSet> = quorums.unwrap().unwrap();

        assert_eq!(quorums.len(), 2);
        assert!(fbas.is_quorum(&quorums[0]));
        assert!(fbas.is_quorum(&quorums[1]));
        assert!(quorums[0].is_disjoint(&quorums[1]));
    }

    #[test]
    fn analysis_with_merging_by_organization_nontrivial() {
        let fbas = Fbas::from_json_file(Path::new("test_data/correct.json"));
        let organizations = Groupings::from_json_str(
            r#"[
            {
                "id": "266107f8966d45eedce41fee2581326d",
                "name": "Stellar Development Foundation",
                "validators": [
                    "GCM6QMP3DLRPTAZW2UZPCPX2LF3SXWXKPMP3GKFZBDSF3QZGV2G5QSTK",
                    "GCGB2S2KGYARPVIA37HYZXVRM2YZUEXA6S33ZU5BUDC6THSB62LZSTYH",
                    "GABMKJM6I25XI4K7U6XWMULOUQIQ27BCTMLS6BYYSOWKTBUXVRJSXHYQ"
                ]
            }]"#,
            &fbas,
        );
        let analysis = Analysis::new(&fbas);

        assert!(analysis.has_quorum_intersection());
        assert_eq!(
            analysis
                .minimal_quorums()
                .merged_by_group(&organizations)
                .minimal_sets()
                .len(),
            1
        );
        assert_eq!(
            analysis
                .minimal_blocking_sets()
                .merged_by_group(&organizations)
                .minimal_sets()
                .len(),
            1
        );
        assert_eq!(
            analysis
                .minimal_splitting_sets()
                .merged_by_group(&organizations)
                .minimal_sets()
                .len(),
            1
        );
    }

    #[test]
    #[ignore]
    fn top_tier_analysis_big() {
        let fbas = Fbas::from_json_file(Path::new("test_data/stellarbeat_nodes_2019-09-17.json"));
        let analysis = Analysis::new(&fbas);

        // calculated with fbas_analyzer v0.1
        let expected = bitset![1, 4, 8, 23, 29, 36, 37, 43, 44, 52, 56, 69, 86, 105, 167, 168, 171];
        let actual = analysis.top_tier().unwrap();
        assert_eq!(expected, actual);
    }

    #[test]
    fn minimal_quorums_id_ordering() {
        let fbas = Fbas::from_json_str(
            r#"[
            {
                "publicKey": "n0",
                "quorumSet": { "threshold": 2, "validators": ["n1", "n2"] }
            },
            {
                "publicKey": "n1",
                "quorumSet": { "threshold": 2, "validators": ["n1", "n2"] }
            },
            {
                "publicKey": "n2",
                "quorumSet": { "threshold": 2, "validators": ["n1", "n2"] }
            }
        ]"#,
        );
        let analysis = Analysis::new(&fbas);
        let expected = vec![bitset![1, 2]];
        let actual = analysis.minimal_quorums().unwrap();
        assert_eq!(expected, actual);
    }
}
