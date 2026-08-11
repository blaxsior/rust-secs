use crate::{
    item::Secs2Variant,
    validate::rule::{CommonRule, ListRule, NodeKind, ScalarRule, ValueRange},
};

pub struct Secs2MessageValidator;

impl Secs2MessageValidator {
    pub fn validate(rule: &CommonRule, msg: Option<&Secs2Variant>) -> bool {
        // root node에 대한 item cardinality -> 0 or 1
        let payload_count = match msg {
            Some(_) => 1,
            None => 0,
        };

        // root에 대한 카디널리티 검사 (required 여부)
        if !Self::validate_range(&rule.cardinality, payload_count) {
            return false;
        }

        // 실질적인 메시지 처리
        match msg {
            None => true, // 이미 required 검사 통과. 없어도 ok
            Some(msg) => Self::validate_node(rule, msg),
        }
    }

    fn validate_node(rule: &CommonRule, msg: &Secs2Variant) -> bool {
        // 아이템 길이 검증
        if !Self::validate_len(rule.len.as_ref(), msg.count()) {
            return false;
        }

        match &rule.kind {
            NodeKind::Scalar(scalar_rule) => Self::validate_scalar(scalar_rule, msg),
            NodeKind::List(list_rule) => Self::validate_list(list_rule, msg),
        }
    }

    fn validate_scalar(rule: &ScalarRule, msg: &Secs2Variant) -> bool {
        // list 아이템은 명확하게 거절
        if matches!(msg, Secs2Variant::List(_)) {
            return false;
        }

        if let Some(allowed_types) = &rule.allowed_types {
            let format = msg.format_code();
            if !allowed_types.contains(&format) {
                return false;
            }
        }

        let Some(_pattern_str) = &rule.pattern else {
            // 패턴 검사 안함
            return true;
        };

        true
    }

    fn validate_list(rule: &ListRule, msg: &Secs2Variant) -> bool {
        // 반드시 list 타입이어야 함
        let Secs2Variant::List(list_msg) = msg else {
            return false;
        };

        Self::match_children(&rule.children, list_msg.items())
    }

    /// list 내 자식들에 대한 매칭을 시도한다.
    fn match_children(rules: &[CommonRule], msgs: &[Secs2Variant]) -> bool {
        let mut rule_idx = 0;
        let mut msg_idx = 0;

        // rule / msg를 two-pointer 방식으로 체크
        while rule_idx < rules.len() && msg_idx < msgs.len() {
            let rule = &rules[rule_idx];
            let mut matched = 0;

            // 남은 메시지를 일괄 검사
            while msg_idx < msgs.len()
                && Self::can_repeat(&rule.cardinality, matched + 1)
                && Self::validate_node(rule, &msgs[msg_idx])
            {
                matched += 1;
                msg_idx += 1;
            }

            // 매칭된 값이 cardinality 조건을 만족하는지 검사
            // 만족 하면 다음 rule로 넘어감
            if Self::validate_range(&rule.cardinality, matched) {
                rule_idx += 1;
                continue;
            }

            return false;
        }

        msg_idx == msgs.len()
            && rules[rule_idx..]
                .iter()
                .all(|rule| Self::validate_range(&rule.cardinality, 0))
    }

    /// value range에 대해 반복 검사 가능한지 체크
    fn can_repeat(rule: &ValueRange, next_count: usize) -> bool {
        next_count <= rule.max
    }

    fn validate_len(rule: Option<&ValueRange>, count: usize) -> bool {
        match rule {
            None => true,
            Some(rule) => Self::validate_range(rule, count),
        }
    }

    fn validate_range(rule: &ValueRange, count: usize) -> bool {
        count >= rule.min && count <= rule.max
    }
}

#[cfg(test)]
mod tests {
    use alloc::vec;

    use super::*;
    use crate::item::Secs2FormatCode;

    fn range(min: usize, max: usize) -> ValueRange {
        ValueRange { min, max }
    }

    fn single_scalar(allowed_types: &[Secs2FormatCode]) -> CommonRule {
        CommonRule {
            cardinality: ValueRange::single(),
            len: None,
            kind: NodeKind::Scalar(ScalarRule {
                allowed_types: Some(allowed_types.to_vec()),
                pattern: None,
            }),
        }
    }

    #[test]
    fn validate_scalar() {
        let rule = CommonRule {
            cardinality: ValueRange::single(),
            len: Some(range(5, 5)),
            kind: NodeKind::Scalar(ScalarRule {
                allowed_types: Some(vec![Secs2FormatCode::ASCII]),
                pattern: None,
            }),
        };

        let msg = Secs2Variant::ascii("HELLO");
        let list_msg = Secs2Variant::list(vec![Secs2Variant::ascii("HELLO")]);

        assert!(Secs2MessageValidator::validate(&rule, Some(&msg)));
        assert!(!Secs2MessageValidator::validate(&rule, None));
        assert!(!Secs2MessageValidator::validate(&rule, Some(&list_msg)));
    }

    #[test]
    fn validate_list() {
        let rule = CommonRule {
            cardinality: ValueRange::single(),
            len: Some(range(2, 2)),
            kind: NodeKind::List(ListRule {
                children: vec![
                    single_scalar(&[Secs2FormatCode::ASCII]),
                    single_scalar(&[Secs2FormatCode::UInt1]),
                ],
            }),
        };

        let msg = Secs2Variant::list(vec![Secs2Variant::ascii("HELLO"), Secs2Variant::uint1(7)]);
        let missing_child = Secs2Variant::list(vec![Secs2Variant::ascii("HELLO")]);
        let wrong_order = Secs2Variant::list(vec![Secs2Variant::uint1(7), Secs2Variant::ascii("HELLO")]);

        assert!(Secs2MessageValidator::validate(&rule, Some(&msg)));
        assert!(!Secs2MessageValidator::validate(&rule, None));
        assert!(!Secs2MessageValidator::validate(&rule, Some(&missing_child)));
        assert!(!Secs2MessageValidator::validate(&rule, Some(&wrong_order)));
    }

    #[test]
    fn validate_list_child_repeat() {
        let mut ascii_rule = single_scalar(&[Secs2FormatCode::ASCII]);
        ascii_rule.cardinality = range(2, 3);

        let rule = CommonRule {
            cardinality: ValueRange::single(),
            len: None,
            kind: NodeKind::List(ListRule {
                children: vec![ascii_rule, single_scalar(&[Secs2FormatCode::UInt1])],
            }),
        };

        let msg = Secs2Variant::list(vec![
            Secs2Variant::ascii("A"),
            Secs2Variant::ascii("B"),
            Secs2Variant::uint1(7),
        ]);
        let too_few = Secs2Variant::list(vec![Secs2Variant::ascii("A"), Secs2Variant::uint1(7)]);
        let too_many = Secs2Variant::list(vec![
            Secs2Variant::ascii("A"),
            Secs2Variant::ascii("B"),
            Secs2Variant::ascii("C"),
            Secs2Variant::ascii("D"),
            Secs2Variant::uint1(7),
        ]);

        assert!(Secs2MessageValidator::validate(&rule, Some(&msg)));
        assert!(!Secs2MessageValidator::validate(&rule, Some(&too_few)));
        assert!(!Secs2MessageValidator::validate(&rule, Some(&too_many)));
    }

    #[test]
    fn validate_nested_list() {
        let nested_list_rule = CommonRule {
            cardinality: ValueRange::single(),
            len: Some(range(2, 2)),
            kind: NodeKind::List(ListRule {
                children: vec![
                    single_scalar(&[Secs2FormatCode::ASCII]),
                    single_scalar(&[Secs2FormatCode::UInt1]),
                ],
            }),
        };

        let rule = CommonRule {
            cardinality: ValueRange::single(),
            len: Some(range(2, 2)),
            kind: NodeKind::List(ListRule {
                children: vec![single_scalar(&[Secs2FormatCode::ASCII]), nested_list_rule],
            }),
        };

        let msg = Secs2Variant::list(vec![
            Secs2Variant::ascii("ROOT"),
            Secs2Variant::list(vec![Secs2Variant::ascii("CHILD"), Secs2Variant::uint1(7)]),
        ]);
        let nested_wrong_order = Secs2Variant::list(vec![
            Secs2Variant::ascii("ROOT"),
            Secs2Variant::list(vec![Secs2Variant::uint1(7), Secs2Variant::ascii("CHILD")]),
        ]);
        let nested_missing_child = Secs2Variant::list(vec![
            Secs2Variant::ascii("ROOT"),
            Secs2Variant::list(vec![Secs2Variant::ascii("CHILD")]),
        ]);

        assert!(Secs2MessageValidator::validate(&rule, Some(&msg)));
        assert!(!Secs2MessageValidator::validate(&rule, Some(&nested_wrong_order)));
        assert!(!Secs2MessageValidator::validate(&rule, Some(&nested_missing_child)));
    }

    #[test]
    fn validate_repeated_nested_list() {
        let nested_list_rule = CommonRule {
            cardinality: range(2, 3),
            len: Some(range(2, 2)),
            kind: NodeKind::List(ListRule {
                children: vec![
                    single_scalar(&[Secs2FormatCode::ASCII]),
                    single_scalar(&[Secs2FormatCode::UInt1]),
                ],
            }),
        };

        let rule = CommonRule {
            cardinality: ValueRange::single(),
            len: None,
            kind: NodeKind::List(ListRule {
                children: vec![nested_list_rule, single_scalar(&[Secs2FormatCode::ASCII])],
            }),
        };

        let msg = Secs2Variant::list(vec![
            Secs2Variant::list(vec![Secs2Variant::ascii("A"), Secs2Variant::uint1(1)]),
            Secs2Variant::list(vec![Secs2Variant::ascii("B"), Secs2Variant::uint1(2)]),
            Secs2Variant::ascii("END"),
        ]);
        let too_few = Secs2Variant::list(vec![
            Secs2Variant::list(vec![Secs2Variant::ascii("A"), Secs2Variant::uint1(1)]),
            Secs2Variant::ascii("END"),
        ]);
        let too_many = Secs2Variant::list(vec![
            Secs2Variant::list(vec![Secs2Variant::ascii("A"), Secs2Variant::uint1(1)]),
            Secs2Variant::list(vec![Secs2Variant::ascii("B"), Secs2Variant::uint1(2)]),
            Secs2Variant::list(vec![Secs2Variant::ascii("C"), Secs2Variant::uint1(3)]),
            Secs2Variant::list(vec![Secs2Variant::ascii("D"), Secs2Variant::uint1(4)]),
            Secs2Variant::ascii("END"),
        ]);
        let invalid_nested = Secs2Variant::list(vec![
            Secs2Variant::list(vec![Secs2Variant::ascii("A"), Secs2Variant::uint1(1)]),
            Secs2Variant::list(vec![Secs2Variant::uint1(2), Secs2Variant::ascii("B")]),
            Secs2Variant::ascii("END"),
        ]);

        assert!(Secs2MessageValidator::validate(&rule, Some(&msg)));
        assert!(!Secs2MessageValidator::validate(&rule, Some(&too_few)));
        assert!(!Secs2MessageValidator::validate(&rule, Some(&too_many)));
        assert!(!Secs2MessageValidator::validate(&rule, Some(&invalid_nested)));
    }
}
