//
// Deterministic TerrainElevation: a table of elevations by cell, as the C
// reference harness supplies get_3d_terrain_point_data ("terrain" line):
// cell (floor (x / size), floor (z / size)), the default outside the grid.
// Every lookup can be observed (in call order) without affecting the result.
//

import type { TerrainElevation } from "../../src/ports";

export class GridTerrainElevation implements TerrainElevation {
	private defaultElevation = 0;

	private cellSize = 1;

	private cellsX = 0;

	private cellsZ = 0;

	private cells: number[] = [];

	private observer: ((x: number, z: number, elevation: number) => void) | undefined = undefined;

	public setGrid(defaultElevation: number, cellSize: number, cellsX: number, cellsZ: number, cells: number[]): void {
		if (cellsX * cellsZ !== cells.length || cellSize <= 0) {
			throw new Error("bad terrain grid");
		}
		this.defaultElevation = defaultElevation;
		this.cellSize = cellSize;
		this.cellsX = cellsX;
		this.cellsZ = cellsZ;
		this.cells = cells.slice();
	}

	public observe(observer: (x: number, z: number, elevation: number) => void): void {
		this.observer = observer;
	}

	public getTerrainElevation(x: number, z: number): number {
		const cx = Math.floor(x / this.cellSize);
		const cz = Math.floor(z / this.cellSize);
		const elevation = cx >= 0 && cx < this.cellsX && cz >= 0 && cz < this.cellsZ ? this.cells[cz * this.cellsX + cx] : this.defaultElevation;

		if (this.observer !== undefined) {
			this.observer(x, z, elevation);
		}

		return elevation;
	}
}
