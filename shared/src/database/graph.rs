#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
	Inbound,
	Outbound,
}

pub trait GraphKey: Send + Sync {
	fn has_next(&self, direction: Direction) -> bool {
		match direction {
			Direction::Inbound => self.has_inbound(),
			Direction::Outbound => self.has_outbound(),
		}
	}

	fn has_inbound(&self) -> bool;
	fn has_outbound(&self) -> bool;
}

pub trait GraphEdge: Send + Sync {
	type Key: GraphKey;

	fn edge_next(&self, direction: Direction) -> impl IntoIterator<Item = Self::Key> + Send;
}

pub trait GraphTraverse: Send + Sync {
	type Edge: GraphEdge;
	type Error;

	fn traversal(
		&self,
		direction: Direction,
		start_nodes: impl IntoIterator<Item = <Self::Edge as GraphEdge>::Key> + Send,
	) -> impl std::future::Future<Output = Result<Vec<Self::Edge>, Self::Error>> + Send
	where
		<Self::Edge as GraphEdge>::Key: std::hash::Hash + std::cmp::Eq + Clone,
	{
		async move {
			let mut visited = fnv::FnvHashSet::default();
			self.traversal_filter(direction, start_nodes, |kind| visited.insert(kind.clone()))
				.await
		}
	}

	fn traversal_filter(
		&self,
		direction: Direction,
		start_nodes: impl IntoIterator<Item = <Self::Edge as GraphEdge>::Key> + Send,
		mut filter: impl FnMut(&<Self::Edge as GraphEdge>::Key) -> bool + Send,
	) -> impl std::future::Future<Output = Result<Vec<Self::Edge>, Self::Error>> + Send {
		async move {
			let mut total_edges = vec![];

			let mut filter_edge = |kind: <Self::Edge as GraphEdge>::Key| {
				if kind.has_next(direction) && filter(&kind) {
					Some(kind)
				} else {
					None
				}
			};

			let mut next_edges = start_nodes.into_iter().filter_map(&mut filter_edge).collect::<Vec<_>>();

			while !next_edges.is_empty() {
				let new_edges = self.fetch_edges(direction, &next_edges).await?;

				next_edges.clear();
				next_edges.extend(
					new_edges
						.iter()
						.flat_map(|edge| edge.edge_next(direction).into_iter())
						.filter_map(&mut filter_edge),
				);
				total_edges.extend(new_edges);
			}

			Ok(total_edges)
		}
	}

	fn fetch_edges(
		&self,
		direction: Direction,
		nodes: &[<Self::Edge as GraphEdge>::Key],
	) -> impl std::future::Future<Output = Result<Vec<Self::Edge>, Self::Error>> + Send;
}
