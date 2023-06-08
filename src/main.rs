fn main() {
    let (width, height, panes) = gather_informations();
    if panes.len() < 2 {
        return;
    }

    let layout = calc_layout(width, height, &panes);
    select_layout(layout);
    swap_pane(&panes);
}

fn get_active(panes: &Vec<(i64, bool)>) -> i64 {
    return panes
        .iter()
        .find(|(_, active)| *active)
        .map(|(id, _)| *id)
        .unwrap();
}

fn get_right(panes: &Vec<(i64, bool)>) -> i64 {
    let mut panes = panes.to_vec();
    panes.sort_by_key(|(idx, _)| *idx);
    return panes.last().map(|(idx, _)| *idx).unwrap();
}

fn swap_pane(panes: &Vec<(i64, bool)>) {
    use tmux_interface::{SelectPane, SwapPane, Tmux};
    let mut panes = panes.to_vec();
    panes.sort_by_key(|(idx, _)| *idx);
    let active = format!("{}", get_active(&panes));
    let right = format!("{}", get_right(&panes));

    let swap_pane = SwapPane::new();
    let swap_pane = swap_pane.src_pane(active);
    let swap_pane = swap_pane.dst_pane(right.clone());

    let select_pane = SelectPane::new();
    let select_pane = select_pane.target_pane(right.clone());
    Tmux::with_command(swap_pane)
        .add_command(select_pane)
        .output()
        .unwrap();
}

fn select_layout(layout: String) {
    use tmux_interface::{SelectLayout, Tmux};
    let select_layout = SelectLayout::new();
    let select_layout = select_layout.layout_name(layout);
    Tmux::with_command(select_layout).output().unwrap();
}

fn gather_informations() -> (i64, i64, Vec<(i64, bool)>) {
    use tmux_interface::{DisplayMessage, ListPanes, Tmux};

    let display_message = DisplayMessage::new()
        .message("#{window_width},#{window_height}")
        .print();
    let sizes = Tmux::with_command(display_message).output().unwrap();
    let sizes = String::from_utf8(sizes.stdout()).unwrap();
    let window_info: Vec<_> = sizes
        .trim()
        .split(",")
        .collect::<Vec<&str>>()
        .iter()
        .map(|&x| x.parse::<i64>().unwrap())
        .collect();
    let (window_width, window_height) = (window_info[0], window_info[1]);

    let list_panes = ListPanes::new().format("#{pane_index},#{pane_active}");
    let panes = Tmux::with_command(list_panes).output().unwrap();
    let panes = String::from_utf8(panes.stdout()).unwrap();
    let panes: Vec<_> = panes
        .trim()
        .split("\n")
        .into_iter()
        .map(|x| x.split(",").collect::<Vec<&str>>())
        .map(|x| {
            (
                x[0].strip_prefix("%")
                    .unwrap_or(x[0])
                    .parse::<i64>()
                    .unwrap(),
                x[1] == "1",
            )
        })
        .collect();

    return (window_width, window_height, panes);
}

fn calc_layout(window_width: i64, window_height: i64, panes: &Vec<(i64, bool)>) -> String {
    use tmux_interface::LayoutChecksum;

    let pane_width = window_width / 2;
    let left_width = if window_width % 2 == 0 {
        pane_width - 1
    } else {
        pane_width
    };
    let right_width = pane_width;
    let left_pane_count = panes.len() as i64 - 1;
    let left_pane_height_sum = window_height - left_pane_count + 1;
    let left_pane_height = left_pane_height_sum / left_pane_count;
    let left_pane_additional_height = left_pane_height_sum - left_pane_height * left_pane_count;

    let window = format!("{}x{},0,0", window_width, window_height);
    let right = format!("{}x{},{},0", right_width, window_height, right_width,);
    let left_parent = format!("{}x{},0,0", left_width, window_height);

    let left_children: Vec<_> = (0..left_pane_count)
        .map(|i| {
            let pane_height = if i < left_pane_additional_height {
                left_pane_height + 1
            } else {
                left_pane_height
            };
            let pane_offset = if i < left_pane_additional_height {
                i * left_pane_height + i
            } else {
                i * left_pane_count + left_pane_additional_height
            };
            return format!("{}x{},0,{}", left_width, pane_height, pane_offset);
        })
        .collect();
    let left = format!("{}[{}]", left_parent, left_children.join(","));

    let layout = format!("{}{{{},{}}}", window, left, right);
    let layout = layout.trim();
    let layout = format!("{:04x},{}", LayoutChecksum::calc(layout), layout);
    return layout;
}
