use cap_project::XY;

/// Bounds of the (possibly zoomed) display rect, expressed as a scale of the
/// unzoomed display: `(0,0)-(1,1)` means no zoom, while a zoomed-in view
/// expands the rect beyond the unit square so the visible viewport becomes a
/// sub-region of the source frame.
#[derive(Debug, PartialEq, Clone, Copy)]
pub struct SegmentBounds {
    pub top_left: XY<f64>,
    pub bottom_right: XY<f64>,
}

impl SegmentBounds {
    pub fn new(top_left: XY<f64>, bottom_right: XY<f64>) -> Self {
        Self {
            top_left,
            bottom_right,
        }
    }

    pub fn default() -> Self {
        SegmentBounds::new(XY::new(0.0, 0.0), XY::new(1.0, 1.0))
    }

    /// Build bounds from a zoom `amount` (>= 1) and a normalized center in
    /// [0, 1] "travel space": 0 pins the visible viewport flush to the
    /// left/top edge, 1 flush to the right/bottom edge, 0.5 centers it.
    /// Inputs are clamped so the resulting viewport is always contained in
    /// the source frame (the geometric in-bounds guarantee that replaced the
    /// old `ensure_cursor_visible*` post-correction).
    pub fn from_amount_center(amount: f64, center: XY<f64>) -> Self {
        let amount = if amount.is_finite() {
            amount.max(1.0)
        } else {
            1.0
        };
        let center = XY::new(clamp_unit(center.x), clamp_unit(center.y));

        // Same formula the easing-based `from_segment_with_cursor_constraint`
        // used: scale about the origin, then translate so `center` maps across
        // the valid travel range.
        let center_diff = XY::new(center.x * amount - center.x, center.y * amount - center.y);

        SegmentBounds::new(
            XY::new(0.0 - center_diff.x, 0.0 - center_diff.y),
            XY::new(amount - center_diff.x, amount - center_diff.y),
        )
    }

    /// Remaps a scalar in [0,1] so values inside the outer `snap_ratio` band
    /// stick to the edges: [r, 1-r] -> [0, 1], clamped.
    pub(crate) fn snap_to_edges(scalar: f64, snap_ratio: f64) -> f64 {
        if snap_ratio <= 0.0 {
            return scalar;
        }
        let lo = snap_ratio;
        let hi = 1.0 - snap_ratio + 0.0001;
        if hi <= lo {
            return 0.5;
        }
        ((scalar - lo) / (hi - lo)).clamp(0.0, 1.0)
    }

    /// Maps a focus position (UV space) to a `from_amount_center`-space center
    /// for auto zooms: edge-snap each axis, then clamp to [0, 1].
    ///
    /// `from_amount_center` places the center scalar PROPORTIONALLY across the
    /// frame (s = 0 is top-left-flush, s = 1 is bottom-right-flush), so the
    /// full [0, 1] range is exactly the set of in-bounds framings and a focus
    /// snapped to 1.0 puts the frame corner-flush with the content corner —
    /// no centered-viewport band, which would make corners unreachable.
    pub(crate) fn calculate_follow_center(
        focus_pos: (f64, f64),
        edge_snap_ratio: f64,
    ) -> (f64, f64) {
        (
            Self::snap_to_edges(focus_pos.0, edge_snap_ratio).clamp(0.0, 1.0),
            Self::snap_to_edges(focus_pos.1, edge_snap_ratio).clamp(0.0, 1.0),
        )
    }
}

fn clamp_unit(v: f64) -> f64 {
    if v.is_finite() {
        v.clamp(0.0, 1.0)
    } else {
        0.5
    }
}

/// A sampled zoom transform. Produced by `ZoomTransformTimeline::sample`.
#[derive(Debug, Clone, Copy)]
pub struct InterpolatedZoom {
    /// Smoothed 0..1 "zoom activity": the spring-driven response to whether
    /// any zoom segment is active. Reaches 1 once a zoom has fully engaged
    /// and returns to 0 at rest. Consumers (e.g. camera scale-during-zoom)
    /// must not assume monotonicity within a segment; it is always clamped
    /// to [0, 1].
    pub t: f64,
    pub bounds: SegmentBounds,
}

impl InterpolatedZoom {
    pub fn display_amount(&self) -> f64 {
        (self.bounds.bottom_right - self.bounds.top_left).x
    }
}

impl Default for InterpolatedZoom {
    /// The resting transform: no zoom engaged.
    fn default() -> Self {
        Self {
            t: 0.0,
            bounds: SegmentBounds::default(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SemanticAttentionEvent {
    pub start_sec: f64,
    pub end_sec: f64,
    pub center: XY<f64>,
    pub suggested_zoom: f64,
    pub priority: u8,
}

pub fn compute_semantic_attention_bounds(
    time_sec: f64,
    events: &[SemanticAttentionEvent],
    fallback_cursor: Option<XY<f64>>,
) -> Option<SegmentBounds> {
    let active = events
        .iter()
        .filter(|ev| time_sec >= ev.start_sec && time_sec <= ev.end_sec)
        .max_by_key(|ev| ev.priority);

    if let Some(ev) = active {
        let duration = (ev.end_sec - ev.start_sec).max(0.05);
        let elapsed = (time_sec - ev.start_sec).clamp(0.0, duration);
        let ramp_in = (elapsed / 0.22).clamp(0.0, 1.0);
        let ramp_out = ((ev.end_sec - time_sec) / 0.22).clamp(0.0, 1.0);
        let smooth = ramp_in.min(ramp_out);
        let spring_t = smooth * smooth * (3.0 - 2.0 * smooth);
        let amount = 1.0 + (ev.suggested_zoom.max(1.0) - 1.0) * spring_t;
        let (fx, fy) = SegmentBounds::calculate_follow_center((ev.center.x, ev.center.y), 0.08);
        return Some(SegmentBounds::from_amount_center(amount, XY::new(fx, fy)));
    }

    fallback_cursor.map(|cursor| {
        let (fx, fy) = SegmentBounds::calculate_follow_center((cursor.x, cursor.y), 0.1);
        SegmentBounds::from_amount_center(1.35, XY::new(fx, fy))
    })
}

#[cfg(test)]
mod test {
    use super::*;

    // The old easing-based tests (segment ramps, ensure_cursor_visible*) were
    // removed together with the state machine they exercised; the spring
    // timeline that replaced it is covered in `zoom_spring.rs`. These tests
    // pin down the pure geometry that survived the rework.

    macro_rules! assert_f64_near {
        ($left:expr, $right:expr, $label:literal) => {
            let left = $left;
            let right = $right;
            assert!(
                (left - right).abs() < 1e-9,
                "{}: left `{:?}` != right `{:?}`",
                $label,
                left,
                right
            )
        };
    }

    #[test]
    fn from_amount_center_identity_at_amount_one() {
        let bounds = SegmentBounds::from_amount_center(1.0, XY::new(0.3, 0.9));
        assert_f64_near!(bounds.top_left.x, 0.0, "top_left.x");
        assert_f64_near!(bounds.top_left.y, 0.0, "top_left.y");
        assert_f64_near!(bounds.bottom_right.x, 1.0, "bottom_right.x");
        assert_f64_near!(bounds.bottom_right.y, 1.0, "bottom_right.y");
    }

    #[test]
    fn from_amount_center_centered() {
        let bounds = SegmentBounds::from_amount_center(2.0, XY::new(0.5, 0.5));
        assert_f64_near!(bounds.top_left.x, -0.5, "top_left.x");
        assert_f64_near!(bounds.top_left.y, -0.5, "top_left.y");
        assert_f64_near!(bounds.bottom_right.x, 1.5, "bottom_right.x");
        assert_f64_near!(bounds.bottom_right.y, 1.5, "bottom_right.y");
    }

    #[test]
    fn from_amount_center_edges_stay_in_bounds() {
        // center 0 -> viewport flush left/top, center 1 -> flush right/bottom.
        for amount in [1.0, 1.5, 2.0, 3.0, 4.5] {
            for c in [0.0, 0.25, 0.5, 0.75, 1.0] {
                let bounds = SegmentBounds::from_amount_center(amount, XY::new(c, c));
                // Display must cover the whole output: top_left <= 0 and
                // bottom_right >= 1 on both axes.
                assert!(bounds.top_left.x <= 1e-9, "tl.x for a={amount} c={c}");
                assert!(bounds.top_left.y <= 1e-9, "tl.y for a={amount} c={c}");
                assert!(
                    bounds.bottom_right.x >= 1.0 - 1e-9,
                    "br.x for a={amount} c={c}"
                );
                assert!(
                    bounds.bottom_right.y >= 1.0 - 1e-9,
                    "br.y for a={amount} c={c}"
                );
            }
        }
    }

    #[test]
    fn from_amount_center_clamps_bad_inputs() {
        let bounds = SegmentBounds::from_amount_center(0.25, XY::new(-4.0, 7.0));
        // amount below 1 clamps to identity scale; centers clamp into [0,1].
        assert_f64_near!(bounds.bottom_right.x - bounds.top_left.x, 1.0, "width");
        let nan_bounds = SegmentBounds::from_amount_center(f64::NAN, XY::new(f64::NAN, 0.5));
        assert!(nan_bounds.top_left.x.is_finite());
        assert!(nan_bounds.bottom_right.y.is_finite());
    }

    #[test]
    fn follow_center_maps_focus_proportionally() {
        // Center focus stays centered.
        let (cx, cy) = SegmentBounds::calculate_follow_center((0.5, 0.5), 0.0);
        assert!((cx - 0.5).abs() < 1e-6);
        assert!((cy - 0.5).abs() < 1e-6);

        // Extreme focus reaches the corner-flush framings so edge/corner
        // content is actually reachable (proportional placement semantics).
        let (left, _) = SegmentBounds::calculate_follow_center((0.0, 0.5), 0.0);
        assert!(left.abs() < 1e-6);
        let (right, _) = SegmentBounds::calculate_follow_center((1.0, 0.5), 0.0);
        assert!((right - 1.0).abs() < 1e-6);

        // A corner-flush framing keeps the frame fully covered at any amount.
        let bounds = SegmentBounds::from_amount_center(2.0, XY::new(1.0, 1.0));
        assert!(bounds.top_left.x <= 0.0 && bounds.bottom_right.x >= 1.0);
    }

    #[test]
    fn display_amount_is_zoom_scale() {
        let zoom = InterpolatedZoom {
            t: 1.0,
            bounds: SegmentBounds::from_amount_center(2.5, XY::new(0.4, 0.6)),
        };
        assert!((zoom.display_amount() - 2.5).abs() < 1e-9);
    }

    #[test]
    fn semantic_attention_director_focuses_on_highest_priority_modal_or_error() {
        let events = [
            SemanticAttentionEvent {
                start_sec: 1.0,
                end_sec: 3.0,
                center: XY::new(0.2, 0.2),
                suggested_zoom: 1.5,
                priority: 60,
            },
            SemanticAttentionEvent {
                start_sec: 1.0,
                end_sec: 3.0,
                center: XY::new(0.8, 0.8),
                suggested_zoom: 1.85,
                priority: 100,
            },
        ];
        let bounds = compute_semantic_attention_bounds(2.0, &events, None).unwrap();
        let width = bounds.bottom_right.x - bounds.top_left.x;
        assert!((width - 1.85).abs() < 1e-6);
        assert!(bounds.top_left.x < -0.5);
    }
}
