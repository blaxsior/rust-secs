use alloc::vec::Vec;

use crate::{
    item::{Secs2Item, Secs2Variant},
    validate::rule::{CommonRule, LengthRule, ListRule, NodeKind, ScalarRule, ValueRange},
};

/// 현재 검사 중인 위치 정보
#[derive(Debug, Clone, Default)]
pub struct ValidateContext {
    pub path: Vec<usize>,
}

/// rule 기반으로 SECS-II 메시지를 검증하는 검증기
pub struct Secs2MessageValidator;

impl Secs2MessageValidator {
    pub fn validate(rule: &CommonRule, msg: &Secs2Variant) -> bool {
        Self::validate_inner(rule, Some(msg), &ValidateContext::default())
    }

    fn validate_inner(
        rule: &CommonRule,
        msg: Option<&Secs2Variant>,
        ctx: &ValidateContext,
    ) -> bool {
        match msg {
            None => !rule.required,
            Some(msg) => {
                if !Self::validate_count(rule.count.as_ref(), msg.count()) {
                    return false;
                }

                match &rule.kind {
                    NodeKind::Scalar(scalar_rule) => Self::validate_scalar(scalar_rule, msg, ctx),
                    NodeKind::List(list_rule) => Self::validate_list(list_rule, msg, ctx),
                }
            }
        }
    }

    fn validate_scalar(rule: &ScalarRule, msg: &Secs2Variant, _ctx: &ValidateContext) -> bool {
        if let Some(allowed_types) = &rule.allowed_types {
            let format = msg.format_code();
            if !allowed_types.contains(&format) {
                return false;
            }
        }

        let _ = rule.pattern.as_ref();
        true
    }

    fn validate_list(rule: &ListRule, msg: &Secs2Variant, ctx: &ValidateContext) -> bool {
        let Secs2Variant::List(list_msg) = msg else {
            return false;
        };

        Self::match_children(&rule.children, list_msg.items(), 0, 0, ctx)
    }

    fn match_children(
        rules: &[CommonRule],
        msgs: &[Secs2Variant],
        rule_idx: usize,
        msg_idx: usize,
        ctx: &ValidateContext,
    ) -> bool {
        if rule_idx == rules.len() {
            return msg_idx == msgs.len();
        }

        let rule = &rules[rule_idx];

        if msg_idx >= msgs.len() {
            return !rule.required
                && Self::match_children(rules, msgs, rule_idx + 1, msg_idx, ctx);
        }

        let mut next_ctx = ctx.clone();
        next_ctx.path.push(msg_idx);

        let consume_ok = Self::validate_inner(rule, Some(&msgs[msg_idx]), &next_ctx);
        if consume_ok {
            if rule.repeated {
                if Self::match_children(rules, msgs, rule_idx, msg_idx + 1, ctx) {
                    return true;
                }
            }

            if Self::match_children(rules, msgs, rule_idx + 1, msg_idx + 1, ctx) {
                return true;
            }
        }

        if !rule.required && Self::match_children(rules, msgs, rule_idx + 1, msg_idx, ctx) {
            return true;
        }

        false
    }

    fn validate_count(rule: Option<&LengthRule>, count: usize) -> bool {
        match rule {
            None => true,
            Some(LengthRule::Exact(exact)) => count == *exact,
            Some(LengthRule::Range(ValueRange { min, max })) => count >= *min && count <= *max,
        }
    }
}
