fn main() {
    let (width, height, panes) = gather_informations();
    let layout = calc_layout(width, height, panes);
    println!("{}", layout)
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

    let list_panes = ListPanes::new().format("#{pane_id},#{pane_active}");
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

fn calc_layout(window_width: i64, window_height: i64, panes: Vec<(i64, bool)>) -> String {
    use tmux_interface::LayoutChecksum;
    let active_pane = panes
        .iter()
        .find(|(_, active)| *active)
        .map(|(id, _)| id)
        .unwrap();
    let pane_width = window_width / 2;
    let pane_height = window_height / (panes.len() as i64 - 1);
    let window = format!("{}x{},0,0", window_width, window_height);
    let right = format!(
        "{}x{},{},0,{}",
        pane_width, window_height, pane_width, active_pane,
    );
    let left_parent = format!("{}x{},0,0", pane_width, window_height);

    let left_children: Vec<_> = panes
        .iter()
        .filter(|(_, active)| !*active)
        .enumerate()
        .map(|(i, (id, _))| {
            format!(
                "{}x{},0,{},{}",
                pane_width,
                pane_height,
                i as i64 * pane_height,
                id
            )
        })
        .collect();
    let left = format!("{}[{}]", left_parent, left_children.join(","));

    let layout = format!("{}{{{},{}}}", window, left, right);
    let layout = layout.trim();
    let layout = format!("{:04x},{}", LayoutChecksum::calc(layout), layout);
    return layout;
}
