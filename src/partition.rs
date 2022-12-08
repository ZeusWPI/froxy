/// See https://jeromekelleher.net/generating-integer-partitions.html
///
/// Generates the integer partitioning of a given argument
fn integer_partitioning(n: usize) -> Vec<Vec<usize>> {
	let mut chunks = vec![];

	let mut a = vec![0; n + 1];
	let mut k = 1;
	let mut y = n - 1;

	while k != 0 {
		let mut x = a[k - 1] + 1;
		k -= 1;

		while 2 * x <= y {
			a[k] = x;
			y -= x;
			k += 1;
		}

		let l = k + 1;
		while x <= y {
			a[k] = x;
			a[l] = y;
			chunks.push(a[..k + 2].to_vec());
			x += 1;
			y -= 1;
		}

		a[k] = x + y;
		y = x + y - 1;
		chunks.push(a[..k + 1].to_vec());
	}

	chunks
}

/// Convert a single partitioning into a series of rectangles
///
/// eg.
/// 10 x 10 screen
/// [
///   [2, 2]
///   [2, 2]
/// ]
/// => (5x5) (5x5)
///    (5x5) (5x5)
fn partitioning_to_rects(
	width: usize,
	height: usize,
	partitioning: &[usize],
) -> Vec<Vec<(usize, usize)>> {
	let mut rects = vec![];

	let rows = partitioning.len();
	let rect_height = (height / rows) as usize;

	for cols in partitioning {
		let rect_width = (width / cols) as usize;

		let mut row_rects = vec![];
		for _ in 0..*cols {
			row_rects.push((rect_width, rect_height));
		}

		rects.push(row_rects);
	}

	rects
}

/// Calculates the "squareness" of a specific partitioning
///
/// SF = (w0-h0)^2 + ... + (wn-hn)^2
///
/// Lower SF => more 'square'
pub(super) fn squareness(v: &[Vec<(usize, usize)>]) -> isize {
	v.iter().flatten().map(|part| (part.0 as isize - part.1 as isize).pow(2)).sum()
}

/// Pretty print a specific partitioning
pub(super) fn print_partitioning(partitioning: &[Vec<(usize, usize)>]) {
	for row in partitioning {
		for rect in row {
			print!("{}x{} ", rect.0, rect.1);
		}
		println!("");
	}
}

/// Find the optimal partitioning for a screen of a given width and height into
/// a given number of sections
pub(super) fn create_partitioning(
	width: usize,
	height: usize,
	sections: usize,
) -> Vec<Vec<(usize, usize)>> {
	let partitioning = integer_partitioning(sections);

	let mut partitioned_rects = partitioning
		.iter()
		.map(|p| partitioning_to_rects(width, height, p))
		.collect::<Vec<Vec<Vec<(usize, usize)>>>>();

	partitioned_rects.sort_by(|a, b| squareness(a).cmp(&squareness(b)));

	partitioned_rects[0].to_vec()
}

/// convert a partitioning into a set of virtual screen coords
pub(super) fn create_virtual_screens(
	partitioning: &[Vec<(usize, usize)>],
) -> Vec<Vec<(usize, usize)>> {
	let mut virtual_screens = vec![];

	let mut running_height = 0;
	for row in partitioning {
		let mut running_width = 0;
		let mut row_virt_screens = vec![];
		for rect in row {
			row_virt_screens.push((running_width as usize, running_height as usize));

			running_width += rect.0;
		}

		running_height += row[0].1;
		virtual_screens.push(row_virt_screens);
	}

	virtual_screens
}

/// Don't even ask
pub(super) fn print_virtual_screens(
	partitioning: &[Vec<(usize, usize)>],
	virt_screens: &[Vec<(usize, usize)>],
) {
	let rows = virt_screens.len();
	let max_cols = virt_screens.iter().map(|row| row.len()).max().unwrap() as f32;

	let height = (rows * 5) + 1;

	let repr_width = (height as f32 * 2.5f32).max((13f32 * max_cols) + 2f32);

	let mut lines: Vec<String> = vec![String::new(); (5 * rows) + 1];

	let mut line_idx = 0;
	for (row, p_row) in virt_screens.iter().zip(partitioning) {
		let cols = row.len() as f32;
		let scaled_width = (repr_width / cols) as usize;

		for (i, (screen, rect)) in row.iter().zip(p_row).enumerate() {
			let top_line_length = scaled_width - 1;
			let spacing = scaled_width - 1;
			let coord_padding = (scaled_width / 2) - 6;
			let dim_padding = (scaled_width / 2) - 5;

			lines[line_idx].push_str(&format!("+{}", "-".repeat(top_line_length)));
			lines[line_idx + 1].push_str(&format!("|{}", " ".repeat(spacing)));
			lines[line_idx + 2].push_str(&format!(
				"|{}({:>4};{:>4}){}",
				" ".repeat(coord_padding),
				screen.0,
				screen.1,
				" ".repeat(coord_padding),
			));
			lines[line_idx + 3].push_str(&format!(
				"|{}{:>4}x{:<4}{}",
				" ".repeat(dim_padding),
				rect.0,
				rect.1,
				" ".repeat(dim_padding),
			));

			if lines[line_idx + 2].len() < scaled_width * (i + 1) {
				let l = lines[line_idx + 2].len();
				lines[line_idx + 2].push_str(&" ".repeat(scaled_width * (i + 1) - l));
			}
			if lines[line_idx + 3].len() < scaled_width * (i + 1) {
				let l = lines[line_idx + 3].len();
				lines[line_idx + 3].push_str(&" ".repeat(scaled_width * (i + 1) - l));
			}

			lines[line_idx + 4].push_str(&format!("|{}", " ".repeat(spacing)));
		}

		lines[line_idx].push('+');
		lines[line_idx + 1].push('|');
		lines[line_idx + 2].push('|');
		lines[line_idx + 3].push('|');
		lines[line_idx + 4].push('|');

		line_idx += 5;
	}

	for _ in virt_screens.last().unwrap() {
		let cols = virt_screens.last().unwrap().len();
		let col_ratio = (max_cols - cols as f32 + 1f32) / max_cols as f32;
		let scaled_width = (repr_width as f32 * col_ratio) as usize;

		lines[line_idx].push_str(&format!("+{}", "-".repeat(scaled_width - 1)));
	}
	lines[line_idx].push('+');

	for line in lines {
		println!("{}", line);
	}
}
