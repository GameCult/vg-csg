use std::fmt::Write;

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    DomainKind, DomainNodeSpec, DomainQuery, FeatureClaimKind, FeatureLoweringPolicy, SelectedCut,
    TriangleChunk, TriangleMesh,
};

pub const GEOMETRY_DOMAIN_SCHEMA: &str = "gamecult.geometry.domain.v1";
pub const GEOMETRY_BUILD_REQUEST_SCHEMA: &str = "gamecult.geometry.build_request.v1";
pub const GEOMETRY_SELECTED_CUT_SCHEMA: &str = "gamecult.geometry.selected_cut.v1";
pub const GEOMETRY_CHUNK_ARTIFACT_SCHEMA: &str = "gamecult.geometry.chunk_artifact.v1";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CultGeometryDomainDocument(
    pub String,
    pub String,
    pub String,
    pub CultGeometryDomainNode,
    pub String,
);

impl CultGeometryDomainDocument {
    pub fn from_spec(
        spec: &DomainNodeSpec,
        source_runtime: impl Into<String>,
        created_at: impl Into<String>,
    ) -> Self {
        Self(
            spec.name.clone(),
            spec.name.clone(),
            source_runtime.into(),
            CultGeometryDomainNode::from_spec(spec),
            created_at.into(),
        )
    }

    pub fn record_key(&self) -> String {
        format!(
            "geometry:domain:{}",
            stable_hash(&[&self.1, &self.2, &self.3.stable_fingerprint()])
        )
    }

    pub fn to_msgpack(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(self)
    }

    pub fn from_msgpack(payload: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        rmp_serde::from_slice(payload)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CultGeometryDomainNode(
    pub String,
    pub String,
    pub Vec<f32>,
    pub Vec<f32>,
    pub u64,
    pub Vec<CultGeometryFeatureClaim>,
    pub Vec<CultGeometryDomainNode>,
);

impl CultGeometryDomainNode {
    pub fn from_spec(spec: &DomainNodeSpec) -> Self {
        Self(
            spec.name.clone(),
            domain_kind_name(spec.kind).to_owned(),
            vec3(spec.frame.translation),
            quat(spec.frame.rotation),
            spec.seed,
            spec.claims
                .iter()
                .map(CultGeometryFeatureClaim::from_spec)
                .collect(),
            spec.children
                .iter()
                .map(CultGeometryDomainNode::from_spec)
                .collect(),
        )
    }

    pub fn stable_fingerprint(&self) -> String {
        let mut parts = vec![
            self.0.clone(),
            self.1.clone(),
            stable_f32_array(&self.2),
            stable_f32_array(&self.3),
            self.4.to_string(),
        ];
        parts.extend(
            self.5
                .iter()
                .map(CultGeometryFeatureClaim::stable_fingerprint),
        );
        parts.extend(
            self.6
                .iter()
                .map(CultGeometryDomainNode::stable_fingerprint),
        );
        stable_join(&parts)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CultGeometryFeatureClaim(
    pub String,
    pub Vec<f32>,
    pub Vec<f32>,
    pub Vec<f32>,
    pub Vec<f32>,
    pub String,
    pub u32,
    pub String,
);

impl CultGeometryFeatureClaim {
    pub fn from_spec(spec: &crate::FeatureClaimSpec) -> Self {
        Self(
            spec.name.clone(),
            vec3(spec.frame.translation),
            quat(spec.frame.rotation),
            vec3(spec.support.center()),
            vec3(spec.support.size()),
            feature_kind_name(spec.kind).to_owned(),
            spec.material.0,
            lowering_policy_name(spec.lowering).to_owned(),
        )
    }

    pub fn stable_fingerprint(&self) -> String {
        stable_join(&[
            self.0.clone(),
            stable_f32_array(&self.1),
            stable_f32_array(&self.2),
            stable_f32_array(&self.3),
            stable_f32_array(&self.4),
            self.5.clone(),
            self.6.to_string(),
            self.7.clone(),
        ])
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CultGeometryBuildRequest(
    pub String,
    pub String,
    pub String,
    pub Vec<f32>,
    pub Vec<f32>,
    pub Vec<f32>,
    pub f32,
    pub f32,
    pub f32,
    pub i32,
    pub i32,
    pub Vec<String>,
    pub Vec<String>,
    pub Vec<String>,
    pub String,
);

impl CultGeometryBuildRequest {
    pub fn from_query(
        request_id: impl Into<String>,
        domain_key: impl Into<String>,
        worker_group: impl Into<String>,
        query: &DomainQuery,
        created_at: impl Into<String>,
    ) -> Self {
        Self(
            request_id.into(),
            domain_key.into(),
            worker_group.into(),
            vec3(query.camera_position),
            vec3(query.frustum.min),
            vec3(query.frustum.max),
            query.viewport_height_px,
            query.vertical_fov_radians,
            query.target_error,
            query.triangle_budget as i32,
            query.collider_budget as i32,
            query
                .semantic_filter
                .iter()
                .map(|kind| domain_kind_name(*kind).to_owned())
                .collect(),
            query
                .requested_chunk_keys
                .iter()
                .map(|key| key.0.clone())
                .collect(),
            query
                .dirty_domain_keys
                .iter()
                .map(|key| key.0.clone())
                .collect(),
            created_at.into(),
        )
    }

    pub fn record_key(&self) -> String {
        format!(
            "geometry:request:{}",
            stable_hash(&[
                &self.1,
                &self.2,
                &stable_f32_array(&self.3),
                &stable_f32_array(&self.4),
                &stable_f32_array(&self.5),
                &stable_f32(self.6),
                &stable_f32(self.7),
                &stable_f32(self.8),
                &self.9.to_string(),
                &self.10.to_string(),
                &stable_join(&self.11),
                &stable_join(&self.12),
                &stable_join(&self.13),
            ])
        )
    }

    pub fn to_msgpack(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(self)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CultGeometrySelectedCutManifest(
    pub String,
    pub String,
    pub Vec<String>,
    pub Vec<String>,
    pub Vec<String>,
    pub Vec<CultGeometryContributionRow>,
);

impl CultGeometrySelectedCutManifest {
    pub fn from_cut(cut: &SelectedCut, request_key: impl Into<String>) -> Self {
        Self(
            cut.id.clone(),
            request_key.into(),
            cut.selected_nodes.iter().map(|key| key.0.clone()).collect(),
            cut.deferred_children
                .iter()
                .map(|key| key.0.clone())
                .collect(),
            cut.fallback_nodes.iter().map(|key| key.0.clone()).collect(),
            cut.diagnostics
                .iter()
                .map(CultGeometryContributionRow::from_row)
                .collect(),
        )
    }

    pub fn record_key(&self) -> String {
        format!(
            "geometry:cut:{}",
            stable_hash(&[
                &self.1,
                &self.0,
                &stable_join(&self.2),
                &stable_join(&self.3),
                &stable_join(&self.4),
            ])
        )
    }

    pub fn to_msgpack(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(self)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CultGeometryContributionRow(
    pub String,
    pub String,
    pub f32,
    pub f32,
    pub f32,
    pub i32,
    pub i32,
    pub i32,
    pub bool,
    pub bool,
    pub bool,
    pub bool,
    pub bool,
);

impl CultGeometryContributionRow {
    pub fn from_row(row: &crate::ContributionRow) -> Self {
        Self(
            row.domain_key.0.clone(),
            domain_kind_name(row.kind).to_owned(),
            row.contribution,
            row.projected_screen_error,
            row.semantic_priority,
            row.estimated_triangle_cost as i32,
            row.child_cost as i32,
            row.remaining_triangle_budget as i32,
            row.requested,
            row.dirty,
            row.selected,
            row.used_fallback,
            row.deferred_by_budget,
        )
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CultGeometryChunkArtifact(
    pub String,
    pub String,
    pub String,
    pub Vec<f32>,
    pub Vec<f32>,
    pub Vec<String>,
    pub Vec<String>,
    pub CultGeometryTriangleMesh,
    pub Option<CultGeometryTriangleMesh>,
    pub i32,
    pub i32,
    pub i32,
    pub u64,
    pub bool,
);

impl CultGeometryChunkArtifact {
    pub fn from_chunk(chunk: &TriangleChunk, cut_key: impl Into<String>) -> Self {
        let manifest = chunk.manifest();
        Self(
            chunk.key.0.clone(),
            cut_key.into(),
            chunk.selected_cut_id.clone(),
            vec3(chunk.bounds.min),
            vec3(chunk.bounds.max),
            chunk
                .source_domain_keys
                .iter()
                .map(|key| key.0.clone())
                .collect(),
            chunk.source_claim_keys.clone(),
            CultGeometryTriangleMesh::from_mesh(&chunk.mesh),
            chunk
                .collider_mesh
                .as_ref()
                .map(CultGeometryTriangleMesh::from_mesh),
            chunk.report.input_brushes as i32,
            chunk.report.candidate_pairs as i32,
            chunk.report.rejected_pairs as i32,
            manifest.transition_hint.stable_clip_seed,
            manifest.transition_hint.supports_parent_child_coexistence,
        )
    }

    pub fn record_key(&self) -> String {
        format!(
            "geometry:chunk:{}",
            stable_hash(&[
                &self.1,
                &self.0,
                &self.2,
                &stable_join(&self.5),
                &stable_join(&self.6),
                &self.7.stable_fingerprint(),
                self.8
                    .as_ref()
                    .map(CultGeometryTriangleMesh::stable_fingerprint)
                    .as_deref()
                    .unwrap_or(""),
            ])
        )
    }

    pub fn to_msgpack(&self) -> Result<Vec<u8>, rmp_serde::encode::Error> {
        rmp_serde::to_vec(self)
    }

    pub fn from_msgpack(payload: &[u8]) -> Result<Self, rmp_serde::decode::Error> {
        rmp_serde::from_slice(payload)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CultGeometryTriangleMesh(
    pub Vec<f32>,
    pub Vec<f32>,
    pub Vec<f32>,
    pub Vec<u32>,
    pub Vec<u32>,
);

impl CultGeometryTriangleMesh {
    pub fn from_mesh(mesh: &TriangleMesh) -> Self {
        Self(
            mesh.positions
                .iter()
                .flat_map(|value| [value.x, value.y, value.z])
                .collect(),
            mesh.normals
                .iter()
                .flat_map(|value| [value.x, value.y, value.z])
                .collect(),
            mesh.uvs
                .iter()
                .flat_map(|value| [value.x, value.y])
                .collect(),
            mesh.indices.clone(),
            mesh.triangle_materials
                .iter()
                .map(|material| material.0)
                .collect(),
        )
    }

    pub fn triangle_count(&self) -> usize {
        self.3.len() / 3
    }

    pub fn stable_fingerprint(&self) -> String {
        stable_hash(&[
            &stable_f32_array(&self.0),
            &stable_f32_array(&self.1),
            &stable_f32_array(&self.2),
            &stable_u32_array(&self.3),
            &stable_u32_array(&self.4),
        ])
    }
}

fn vec3(value: bevy_math::Vec3) -> Vec<f32> {
    vec![value.x, value.y, value.z]
}

fn quat(value: bevy_math::Quat) -> Vec<f32> {
    vec![value.x, value.y, value.z, value.w]
}

fn domain_kind_name(kind: DomainKind) -> &'static str {
    match kind {
        DomainKind::Root => "Root",
        DomainKind::Column => "Column",
        DomainKind::AltitudeBand => "AltitudeBand",
        DomainKind::RoadSpine => "RoadSpine",
        DomainKind::BranchRoad => "BranchRoad",
        DomainKind::Roundabout => "Roundabout",
        DomainKind::SupportMass => "SupportMass",
        DomainKind::ClearanceVolume => "ClearanceVolume",
        DomainKind::Chunk => "Chunk",
    }
}

fn feature_kind_name(kind: FeatureClaimKind) -> &'static str {
    match kind {
        FeatureClaimKind::SolidBrush => "SolidBrush",
        FeatureClaimKind::VoidBrush => "VoidBrush",
        FeatureClaimKind::RoadSurfaceSlab => "RoadSurfaceSlab",
        FeatureClaimKind::ClearanceVolume => "ClearanceVolume",
        FeatureClaimKind::SupportShell => "SupportShell",
        FeatureClaimKind::ColliderProxy => "ColliderProxy",
    }
}

fn lowering_policy_name(policy: FeatureLoweringPolicy) -> &'static str {
    match policy {
        FeatureLoweringPolicy::RenderOnly => "RenderOnly",
        FeatureLoweringPolicy::ColliderOnly => "ColliderOnly",
        FeatureLoweringPolicy::RenderAndCollider => "RenderAndCollider",
        FeatureLoweringPolicy::BooleanOperator => "BooleanOperator",
    }
}

fn stable_hash(parts: &[&str]) -> String {
    let canonical = parts.join("\u{1f}");
    let mut hasher = Sha256::new();
    hasher.update(canonical.as_bytes());
    let digest = hasher.finalize();
    let mut output = String::with_capacity(digest.len() * 2);
    for byte in digest {
        write!(&mut output, "{byte:02x}").expect("write to string cannot fail");
    }
    output
}

fn stable_join(parts: &[String]) -> String {
    parts.join("\u{1e}")
}

fn stable_f32_array(values: &[f32]) -> String {
    values
        .iter()
        .map(|value| stable_f32(*value))
        .collect::<Vec<_>>()
        .join(",")
}

fn stable_u32_array(values: &[u32]) -> String {
    values
        .iter()
        .map(u32::to_string)
        .collect::<Vec<_>>()
        .join(",")
}

fn stable_f32(value: f32) -> String {
    format!("{:08x}", value.to_bits())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Aabb, DomainQuery, build_domain_chunks, ragnarok_column_spec};
    use bevy_math::Vec3;

    fn query() -> DomainQuery {
        DomainQuery {
            camera_position: Vec3::new(36.0, -42.0, 30.0),
            frustum: Aabb::from_center_size(Vec3::new(0.0, 0.0, 45.0), Vec3::splat(150.0)),
            viewport_height_px: 1080.0,
            vertical_fov_radians: std::f32::consts::FRAC_PI_3,
            target_error: 0.01,
            triangle_budget: 10_000,
            collider_budget: 10_000,
            semantic_filter: Vec::new(),
            requested_chunk_keys: Vec::new(),
            dirty_domain_keys: Vec::new(),
        }
    }

    fn v1_domain_witness() -> CultGeometryDomainDocument {
        CultGeometryDomainDocument(
            "fixture-domain".to_owned(),
            "fixture/root".to_owned(),
            "vg-csg-a3197f4".to_owned(),
            CultGeometryDomainNode(
                "root".to_owned(),
                "Root".to_owned(),
                vec![1.0, -2.5, 0.0],
                vec![0.0, 0.0, 0.0, 1.0],
                42,
                Vec::new(),
                Vec::new(),
            ),
            "2026-05-29T00:00:00Z".to_owned(),
        )
    }

    fn v1_chunk_witness() -> CultGeometryChunkArtifact {
        CultGeometryChunkArtifact(
            "fixture/chunk".to_owned(),
            "geometry:cut:fixture".to_owned(),
            "cut-fixture".to_owned(),
            vec![-1.0, -2.0, -3.0],
            vec![1.0, 2.0, 3.0],
            vec!["fixture/root".to_owned()],
            vec!["fixture/root/claim".to_owned()],
            CultGeometryTriangleMesh(
                vec![0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0],
                vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
                vec![0.0, 0.0, 1.0, 0.0, 0.0, 1.0],
                vec![0, 1, 2],
                vec![7],
            ),
            None,
            1,
            0,
            0,
            0x0123_4567_89ab_cdef,
            true,
        )
    }

    #[test]
    fn v1_domain_messagepack_and_record_key_are_exact_witnesses() {
        let domain = v1_domain_witness();
        assert_eq!(
            domain.record_key(),
            "geometry:domain:0e38aabb0f3547fb5cdac61351781187202bc493e7c090565996a4f3cca65faa"
        );
        assert_eq!(
            domain.to_msgpack().unwrap(),
            [
                149, 174, 102, 105, 120, 116, 117, 114, 101, 45, 100, 111, 109, 97, 105, 110, 172,
                102, 105, 120, 116, 117, 114, 101, 47, 114, 111, 111, 116, 174, 118, 103, 45, 99,
                115, 103, 45, 97, 51, 49, 57, 55, 102, 52, 151, 164, 114, 111, 111, 116, 164, 82,
                111, 111, 116, 147, 202, 63, 128, 0, 0, 202, 192, 32, 0, 0, 202, 0, 0, 0, 0, 148,
                202, 0, 0, 0, 0, 202, 0, 0, 0, 0, 202, 0, 0, 0, 0, 202, 63, 128, 0, 0, 42, 144,
                144, 180, 50, 48, 50, 54, 45, 48, 53, 45, 50, 57, 84, 48, 48, 58, 48, 48, 58, 48,
                48, 90,
            ]
        );
    }

    #[test]
    fn v1_chunk_messagepack_and_record_key_are_exact_witnesses() {
        let chunk = v1_chunk_witness();
        assert_eq!(
            chunk.record_key(),
            "geometry:chunk:ca2874d3638800179c906087ec817ea715c43b70b99ead04edccce2a1b3d6ebb"
        );
        assert_eq!(
            chunk.to_msgpack().unwrap(),
            [
                158, 173, 102, 105, 120, 116, 117, 114, 101, 47, 99, 104, 117, 110, 107, 180, 103,
                101, 111, 109, 101, 116, 114, 121, 58, 99, 117, 116, 58, 102, 105, 120, 116, 117,
                114, 101, 171, 99, 117, 116, 45, 102, 105, 120, 116, 117, 114, 101, 147, 202, 191,
                128, 0, 0, 202, 192, 0, 0, 0, 202, 192, 64, 0, 0, 147, 202, 63, 128, 0, 0, 202, 64,
                0, 0, 0, 202, 64, 64, 0, 0, 145, 172, 102, 105, 120, 116, 117, 114, 101, 47, 114,
                111, 111, 116, 145, 178, 102, 105, 120, 116, 117, 114, 101, 47, 114, 111, 111, 116,
                47, 99, 108, 97, 105, 109, 149, 153, 202, 0, 0, 0, 0, 202, 0, 0, 0, 0, 202, 0, 0,
                0, 0, 202, 63, 128, 0, 0, 202, 0, 0, 0, 0, 202, 0, 0, 0, 0, 202, 0, 0, 0, 0, 202,
                63, 128, 0, 0, 202, 0, 0, 0, 0, 153, 202, 0, 0, 0, 0, 202, 0, 0, 0, 0, 202, 63,
                128, 0, 0, 202, 0, 0, 0, 0, 202, 0, 0, 0, 0, 202, 63, 128, 0, 0, 202, 0, 0, 0, 0,
                202, 0, 0, 0, 0, 202, 63, 128, 0, 0, 150, 202, 0, 0, 0, 0, 202, 0, 0, 0, 0, 202,
                63, 128, 0, 0, 202, 0, 0, 0, 0, 202, 0, 0, 0, 0, 202, 63, 128, 0, 0, 147, 0, 1, 2,
                145, 7, 192, 1, 0, 0, 207, 1, 35, 69, 103, 137, 171, 205, 239, 195,
            ]
        );
    }

    #[test]
    fn domain_document_emits_msgpack_payload_and_stable_key() {
        let spec = ragnarok_column_spec();
        let document =
            CultGeometryDomainDocument::from_spec(&spec, "vg-csg", "2026-05-29T00:00:00Z");
        let payload = document.to_msgpack().unwrap();
        let decoded = CultGeometryDomainDocument::from_msgpack(&payload).unwrap();
        assert_eq!(document.record_key(), decoded.record_key());
        assert_eq!(
            document.record_key(),
            "geometry:domain:02c9b5810977406b0c206f3a3494327a423abb9448192be1b9a1863cd0f2ed95"
        );
        assert!(document.record_key().starts_with("geometry:domain:"));
        assert!(!payload.is_empty());
    }

    #[test]
    fn chunk_artifact_emits_msgpack_payload_and_stable_key() {
        let spec = ragnarok_column_spec();
        let build = build_domain_chunks(&spec.compile_root(), &query());
        let request = CultGeometryBuildRequest::from_query(
            "request-0",
            "geometry:domain:test",
            "ragnarok-workers",
            &query(),
            "2026-05-29T00:00:00Z",
        );
        let cut = CultGeometrySelectedCutManifest::from_cut(&build.cut, request.record_key());
        let artifact = CultGeometryChunkArtifact::from_chunk(&build.chunks[0], cut.record_key());
        let payload = artifact.to_msgpack().unwrap();
        let decoded = CultGeometryChunkArtifact::from_msgpack(&payload).unwrap();
        assert_eq!(artifact.record_key(), decoded.record_key());
        assert_eq!(
            request.record_key(),
            "geometry:request:87caae7d0d33ada85ff1b05371cd64d48b235628771f15bcc10aed74548eb692"
        );
        assert_eq!(
            cut.record_key(),
            "geometry:cut:6887572d3deb49861196cc407dbfdae64ccd2ad39445e6e2892694b3933f3a92"
        );
        assert_eq!(
            artifact.record_key(),
            "geometry:chunk:e14d7af94449490fbd747f9a24dad40f76472ac2151373881c5ed06db9f007c7"
        );
        assert!(artifact.record_key().starts_with("geometry:chunk:"));
        assert!(decoded.7.triangle_count() > 0);
    }
}
