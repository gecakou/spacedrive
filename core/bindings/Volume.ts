export interface Volume {
	name: string;
	mount_point: string;
	total_capacity: bigint;
	available_capacity: bigint;
	is_removable: boolean;
	disk_type: string | null;
	file_system: string | null;
	is_root_filesystem: boolean;
}
