use log::info;
use num_rational::Ratio;
use psutil::memory;
use size::Size;
use tui::buffer::Buffer;
use tui::layout::Rect;
use tui::widgets::{Axis, Chart, Dataset, GraphType, Marker, Widget};

use crate::colorscheme::Colorscheme;
use crate::update::UpdatableWidget;
use crate::widgets::block;

const HORIZONTAL_SCALE_DELTA: u64 = 25;

#[derive(Default)]
struct MemData {
	total: u64,
	used: u64,
	percents: Vec<(f64, f64)>,
}

pub struct MemWidget<'a> {
	title: String,
	update_interval: Ratio<u64>,
	colorscheme: &'a Colorscheme,

	horizontal_scale: u64,

	update_count: u64,

	main: MemData,
	swap: MemData,
}

impl MemWidget<'_> {
	pub fn new(colorscheme: &Colorscheme, update_interval: Ratio<u64>) -> MemWidget {
		let update_count = 0;

		let mut main = MemData::default();
		let mut swap = MemData::default();

		main.percents.push((update_count as f64, 0.0));
		swap.percents.push((update_count as f64, 0.0));

		MemWidget {
			title: " Memory Usage ".to_string(),
			update_interval,
			colorscheme,

			horizontal_scale: 100,

			update_count,

			main,
			swap,
		}
	}

	pub fn scale_in(&mut self) {
		if self.horizontal_scale > HORIZONTAL_SCALE_DELTA {
			self.horizontal_scale -= HORIZONTAL_SCALE_DELTA;
		}
	}

	pub fn scale_out(&mut self) {
		self.horizontal_scale += HORIZONTAL_SCALE_DELTA;
	}
}

impl UpdatableWidget for MemWidget<'_> {
	fn update(&mut self) {
		self.update_count += 1;

		let main = memory::virtual_memory().unwrap();
		let swap = memory::swap_memory().unwrap();

		self.main.total = main.total();
		self.main.used = main.used();
		self.main
			.percents
			.push((self.update_count as f64, main.percent().into()));

		self.swap.total = swap.total();
		self.swap.used = swap.used();
		self.swap
			.percents
			.push((self.update_count as f64, swap.percent().into()));

		info!("Mem update: main: {:?}", &self.main.percents);
		info!("Mem update: swap: {:?}", &self.swap.percents);
	}

	fn get_update_interval(&self) -> Ratio<u64> {
		self.update_interval
	}
}

impl Widget for MemWidget<'_> {
	fn draw(&mut self, area: Rect, buf: &mut Buffer) {
		info!("Mem draw: main: {:?}", &self.main.percents);
		info!("Mem draw: swap: {:?}", &self.swap.percents);
		Chart::<String, String>::default()
			.block(block::new(self.colorscheme, &self.title))
			.x_axis(Axis::default().bounds([
				self.update_count as f64 - self.horizontal_scale as f64,
				self.update_count as f64 + 1.0,
			]))
			.y_axis(Axis::default().bounds([0.0, 100.0]))
			.datasets(&[
				Dataset::default()
					.marker(Marker::Braille)
					.graph_type(GraphType::Line)
					.style(self.colorscheme.mem_main)
					.data(&self.main.percents),
				Dataset::default()
					.marker(Marker::Braille)
					.graph_type(GraphType::Line)
					.style(self.colorscheme.mem_swap)
					.data(&self.swap.percents),
			])
			.draw(area, buf);

		buf.set_string(
			area.x + 3,
			area.y + 2,
			format!(
				"Main {:3.0}% {}/{}",
				self.main.percents.last().unwrap().1,
				Size::Bytes(self.main.used),
				Size::Bytes(self.main.total),
			),
			self.colorscheme.mem_main,
		);

		buf.set_string(
			area.x + 3,
			area.y + 3,
			format!(
				"Swap {:3.0}% {}/{}",
				self.swap.percents.last().unwrap().1,
				Size::Bytes(self.swap.used),
				Size::Bytes(self.swap.total),
			),
			self.colorscheme.mem_swap,
		);
	}
}
