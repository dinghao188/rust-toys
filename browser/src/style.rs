use crate::css;
use crate::dom;
use crate::layout::Display;

type PropertyMap = std::collections::HashMap<String, css::Value>;

pub struct StyledNode<'a> {
    pub node: &'a dom::Node,
    pub specified_values: PropertyMap,
    pub children: Vec<StyledNode<'a>>,
}

fn matches(elem: &dom::ElementData, selector: &css::Selector) -> bool {
    match selector {
        css::Selector::Simple(simple_selector) => matches_simple_selector(elem, simple_selector),
        _ => false
    }
}

fn matches_simple_selector(elem: &dom::ElementData, selector: &css::SimpleSelector) -> bool {
    // if selector.tag_name not equal to elem.tag_name; it means dismatching
    if selector.tag_name.iter().any(|name| elem.tag_name != *name) {
        return false;
    }

    // if selector.id not equal to elem's id attr; it means dismatching
    if selector.id.iter().any(|id| elem.id() != Some(id)) {
        return false;
    }

    // if any class in selector isn't contained by elem's classes; it means dismatching
    let elem_classes = elem.classes();
    if selector.class.iter().any(|_class| !elem_classes.contains(&**_class)) {
        return false;
    }
    
    return true;
}

type MatchedRule<'a> = (css::Specificity, &'a css::Rule);

fn match_rule<'a>(elem: &dom::ElementData, rule: &'a css::Rule) -> Option<MatchedRule<'a>> {
    // this means if we can find a selector that matches with elem from this rule's selectors
    rule.selectors.iter()
        .find(|selector| matches(elem, selector))
        .map(|selector| (selector.specificity(), rule))
}

fn matching_rules<'a>(elem: &dom::ElementData, style_sheet: &'a css::StyleSheet) -> Vec<MatchedRule<'a>> {
    style_sheet.rules.iter().filter_map(|rule| match_rule(elem, rule)).collect()
}

pub fn style_tree<'a>(root: &'a dom::Node, style_sheet: &'a css::StyleSheet) -> StyledNode<'a> {
    StyledNode {
        node: root,
        specified_values: match &root.node_type {
            dom::NodeType::Element(elem) => specified_values(elem, style_sheet),
            dom::NodeType::Text(_) => PropertyMap::new()
        },
        children: root.children.iter().map(|child_node| style_tree(child_node, style_sheet)).collect(),
    }
}

fn specified_values(elem: &dom::ElementData, style_sheet: &css::StyleSheet) -> PropertyMap {
    let mut values = PropertyMap::new();
    let mut rules = matching_rules(elem, style_sheet);

    rules.sort_by(|&(a, _), &(b, _)| a.cmp(&b));
    for (_, rule) in rules {
        for declaration in &rule.declarations {
            values.insert(declaration.name.clone(), declaration.value.clone());
        }
    }
    
    values
}
