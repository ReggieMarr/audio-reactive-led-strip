use plotlib::page::Page;
use plotlib::scatter;
use plotlib::scatter::Scatter;
use plotlib::view::ContinuousView;
use plotlib::style::{Marker, Point};
// use plotlib::style::{PointMarker, PointStyle};

fn plot_data(data : Vec<(f64, f64)>)->std::io::Result<()> {
    // Scatter plots expect a list of pairs
    // We create our scatter plot from the data
    let s1 = Scatter::from_slice(data.as_slice()).style(
            scatter::Style::new()
            .marker(Marker::Square) // setting the marker to be a square
            .colour("#DD3355"),
    ); // and a custom colour

    // We can plot multiple data sets in the same view
    // let data2 = [(-1.4, 2.5), (7.2, -0.3)];
    // let s2 = Scatter::from_slice(&data2).style(
    //     PointStyle::new() // uses the default marker
    //         .colour("#35C788"),
    // ); // and a different colour

    // The 'view' describes what set of data is drawn
    let v = ContinuousView::new()
        .add(&s1)
        .x_range(-5., 10.)
        .y_range(-2., 6.)
        .x_label("Some varying variable")
        .y_label("The response of something");

    // A page with a single view is then saved to an SVG file
    Page::single(&v).save("scatter.svg").unwrap();
    Ok(())
}