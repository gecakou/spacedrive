// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { DirectoryWithContents } from './DirectoryWithContents';
import type { JobReport } from './JobReport';
import type { LibraryConfigWrapped } from './LibraryConfigWrapped';
import type { LocationResource } from './LocationResource';
import type { NodeState } from './NodeState';
import type { Statistics } from './Statistics';
import type { Tag } from './Tag';
import type { TagWithFiles } from './TagWithFiles';
import type { Volume } from './Volume';

export type CoreResponse =
	| { key: 'Success'; data: null }
	| { key: 'Error'; data: string }
	| { key: 'GetLibraries'; data: Array<LibraryConfigWrapped> }
	| { key: 'GetVolumes'; data: Array<Volume> }
	| { key: 'GetLocation'; data: LocationResource }
	| { key: 'GetLocations'; data: Array<LocationResource> }
	| { key: 'GetExplorerDir'; data: DirectoryWithContents }
	| { key: 'GetNode'; data: NodeState }
	| { key: 'LocCreate'; data: LocationResource }
	| { key: 'GetRunningJobs'; data: Array<JobReport> }
	| { key: 'GetJobHistory'; data: Array<JobReport> }
	| { key: 'GetLibraryStatistics'; data: Statistics };
