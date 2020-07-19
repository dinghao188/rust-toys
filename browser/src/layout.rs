use crate::style::StyledNode;
use crate::css;

#[derive(Default)]
struct Dimensions {
    content: Rect,

    padding: EdgeSizes,
    border: EdgeSizes,
    margin: EdgeSizes,
}

#[derive(Default)]
struct Rect {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
}

#[derive(Default)]
struct EdgeSizes {
    top: f32,
    right: f32,
    bottom: f32,
    left: f32,
}

struct LayoutBox<'a> {
    dimensions: Dimensions,
    box_type: BoxType<'a>,
    children: Vec<LayoutBox<'a>>,
}

impl<'a> LayoutBox<'a> {
    fn new(box_type: BoxType) -> LayoutBox {
        LayoutBox {
            box_type: box_type,
            dimensions: Default::default(),
            children: Vec::new(),
        }
    }

    fn get_inline_container(&mut self) -> &mut LayoutBox<'a> {
        match self.box_type {
            BoxType::InlineNode(_) | BoxType::AnonymousBlock => self,
            BoxType::BlockNode(_) => {
                match self.children.last() {
                    Some(&LayoutBox {box_type: BoxType::AnonymousBlock,..}) => {},
                    _ => self.children.push(LayoutBox::new(BoxType::AnonymousBlock))
                }
                self.children.last_mut().unwrap()
            }
        }
    }

    fn get_style_node(&self) -> &'a StyledNode<'a> {
        match self.box_type {
            BoxType::BlockNode(node) | BoxType::InlineNode(node) => node,
            BoxType::AnonymousBlock => panic!("Anonymous block box has no style node")
        }
    }
}

enum BoxType<'a> {
    BlockNode(&'a StyledNode<'a>),
    InlineNode(&'a StyledNode<'a>),
    AnonymousBlock,
}

pub enum Display {
    Inline,
    Block,
    None,
}

impl StyledNode<'_> {
    fn value(&self, name: &str) -> Option<css::Value> {
        self.specified_values.get(name).map(|v| v.clone())
    }
    fn display(&self) -> Display {
        match self.value("display") {
            Some(css::Value::Keyword(s)) => match &s[..]{
                "block" => Display::Block,
                "none" => Display::Inline,
                _ => Display::Inline
            }
            _ => Display::Inline
        }
    }
}

fn build_layout_tree<'a>(style_node: &'a StyledNode<'a>) -> LayoutBox<'a> {
    let mut root = LayoutBox::new(match style_node.display() {
        Display::Block => BoxType::BlockNode(style_node),
        Display::Inline => BoxType::InlineNode(style_node),
        Display::None => BoxType::AnonymousBlock,
    });

    for child in &style_node.children {
        match child.display() {
            Display::Block => root.children.push(build_layout_tree(child)),
            Display::Inline => root.children.push(build_layout_tree(child)),
            Display::None => root.children.push(build_layout_tree(child)),
        }
    }

    root
}

impl<'a> LayoutBox<'a> {
    fn layout(&mut self, containing_block: Dimensions) {
        match self.box_type {
            BoxType::BlockNode(_) => self.layout_block(containing_block),
            BoxType::InlineNode(_) => {}
            BoxType::AnonymousBlock => {}
        }
    }

    fn layout_block(&mut self, containing_block: Dimensions) {
        self.calculate_block_width(containing_block);

        self.calculate_block_position(containing_block);

        self.layout_block_children(containing_block);

        self.calculate_block_height(containing_block);
    }

    fn calculate_block_width(&mut self, containing_block: Dimensions) {
        let style = self.get_style_node();

        // 'width' initial value to 'auto'
        let auto = css::Value::Keyword("auto".to_string());
        let mut width = style.value("width").unwrap_or(auto.clone());

        let zero = css::Value::Length(0.0, css::Unit::Px);


    }
    fn calculate_block_height(&mut self, containing_block: Dimensions) {
    }
    fn calculate_block_position(&mut self, containing_block: Dimensions) {
    }
    fn layout_block_children(&mut self, containing_block: Dimensions) {
    }

}
