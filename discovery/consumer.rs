// Independent, synthetic SDK consumer. This file has no privileged/internal
// module access and runs only against the real ores-chat-discovery crate.
use ores_chat_discovery::{
    AnalysisAudience, AnalysisPlan, Cohort, CohortScope, Comparison, DiscoveryError, Feature,
    Inference, Interpretation, MissingData, Observation, analyze,
};

fn scope(audience: AnalysisAudience) -> CohortScope {
    CohortScope::new(
        "synthetic-tenant".into(),
        "synthetic-cohort".into(),
        [41; 32],
        audience,
    )
    .unwrap()
}

fn cohort(missing: usize) -> Cohort {
    let owner = scope(AnalysisAudience::Owner);
    let rows = (-10..=10)
        .enumerate()
        .map(|(index, integer)| {
            let x = f64::from(integer);
            let residual = x * x - 110.0 / 3.0;
            Observation::new(
                owner.clone(),
                format!("synthetic-{index:03}"),
                vec![
                    Some(x),
                    if index < missing {
                        None
                    } else {
                        Some(5.0 + 2.0 * x + residual)
                    },
                    Some(-7.0 + 3.0 * x - residual),
                ],
            )
            .unwrap()
        })
        .collect();
    Cohort::new(
        owner,
        ["x", "y", "z"]
            .map(|name| Feature::new(name.into(), "synthetic-units".into()).unwrap())
            .to_vec(),
        rows,
    )
    .unwrap()
}

fn plan(data: &Cohort, family: &[(&str, &str)], missing: MissingData) -> AnalysisPlan {
    AnalysisPlan::new(
        scope(AnalysisAudience::Owner),
        data.fingerprint(),
        family
            .iter()
            .map(|&(x, y)| Comparison::new(x.into(), y.into()).unwrap())
            .collect(),
        missing,
        10,
        0.05,
    )
    .unwrap()
}

fn close(actual: f64, expected: f64, tolerance: f64) {
    assert!((actual - expected).abs() <= tolerance);
}

#[test]
fn consumer_recovers_closed_form_regression_and_labels_it_noncausal() {
    let data = cohort(0);
    let result = analyze(&data, &plan(&data, &[("x", "y")], MissingData::Reject)).unwrap();
    let fit = &result.results[0];
    close(fit.slope, 2.0, 1.0e-13);
    close(fit.intercept, 5.0, 1.0e-13);
    // For integers -10..10: sum(x^2)=770 and sum(x^4)=50666.
    let expected_sse = 50666.0 - 770.0 * 770.0 / 21.0;
    close(fit.residual_sum_squares, expected_sse, 1.0e-9);
    close(fit.r_squared, 3080.0 / (3080.0 + expected_sse), 1.0e-13);
    close(fit.pearson_r.powi(2), fit.r_squared, 1.0e-13);
    assert_eq!(result.observations_used, 21);
    assert_eq!(
        result.interpretation,
        Interpretation::ExploratoryAssociationOnly
    );
}

#[test]
fn consumer_cannot_reuse_owner_cohort_as_administrator_cohort() {
    let data = cohort(0);
    let wrong = AnalysisPlan::new(
        scope(AnalysisAudience::Administrator),
        data.fingerprint(),
        vec![Comparison::new("x".into(), "y".into()).unwrap()],
        MissingData::Reject,
        10,
        0.05,
    )
    .unwrap();
    assert_eq!(
        analyze(&data, &wrong).unwrap_err(),
        DiscoveryError::ScopeMismatch
    );
}

#[test]
fn consumer_cannot_replay_a_plan_over_changed_data() {
    let first = cohort(0);
    let changed = cohort(1);
    assert_eq!(
        analyze(
            &changed,
            &plan(&first, &[("x", "y")], MissingData::CompleteCases)
        )
        .unwrap_err(),
        DiscoveryError::SnapshotMismatch
    );
}

#[test]
fn consumer_observes_missingness_and_post_exclusion_threshold() {
    let permitted = cohort(3);
    assert_eq!(
        analyze(
            &permitted,
            &plan(&permitted, &[("x", "y")], MissingData::Reject)
        )
        .unwrap_err(),
        DiscoveryError::MissingObservation
    );
    let result = analyze(
        &permitted,
        &plan(
            &permitted,
            &[("x", "y"), ("x", "z")],
            MissingData::CompleteCases,
        ),
    )
    .unwrap();
    assert_eq!(result.observations_used, 18);
    assert_eq!(result.observations_excluded, 3);
    let suppressed = cohort(12);
    assert_eq!(
        analyze(
            &suppressed,
            &plan(&suppressed, &[("x", "y")], MissingData::CompleteCases)
        )
        .unwrap_err(),
        DiscoveryError::InsufficientCohort
    );
}

#[test]
fn consumer_receives_family_corrected_inference_not_unadjusted_discovery_claims() {
    let data = cohort(0);
    let one = analyze(&data, &plan(&data, &[("x", "y")], MissingData::Reject)).unwrap();
    let two = analyze(
        &data,
        &plan(&data, &[("x", "y"), ("x", "z")], MissingData::Reject),
    )
    .unwrap();
    match (&one.results[0].inference, &two.results[0].inference) {
        (
            Inference::Estimated {
                two_sided_p: p1,
                bonferroni_slope_interval: first,
                ..
            },
            Inference::Estimated {
                two_sided_p: p2,
                bonferroni_p,
                bonferroni_slope_interval: second,
                ..
            },
        ) => {
            close(*p1, *p2, 0.0);
            close(*bonferroni_p, (2.0 * p1).min(1.0), 1.0e-14);
            assert!(second.lower < first.lower && second.upper > first.upper);
        }
        _ => panic!("synthetic residuals have positive variance"),
    }
    assert_ne!(
        one.provenance.analysis_fingerprint,
        two.provenance.analysis_fingerprint
    );
}

#[test]
fn consumer_diagnostics_do_not_serialize_cohort_or_estimates() {
    let data = cohort(0);
    let plan = plan(&data, &[("x", "y")], MissingData::Reject);
    let report = analyze(&data, &plan).unwrap();
    let debug = format!("{data:?} {plan:?} {report:?} {:?}", report.results[0]);
    assert!(!debug.contains("synthetic"));
    assert!(!debug.contains("slope"));
    assert_eq!(debug.matches("[redacted]").count(), 4);
}
