/* eslint-disable */
// This file was generated by [rspc](https://github.com/oscartbeaumont/rspc). Do not edit this file manually.

export type Procedures = {
    queries: 
        { key: "auth.me", input: never, result: { id: string; email: string } } | 
        { key: "backups.getAll", input: never, result: GetAll } | 
        { key: "buildInfo", input: never, result: BuildInfo } | 
        { key: "cloud.getApiOrigin", input: never, result: string } | 
        { key: "cloud.library.get", input: LibraryArgs<null>, result: { id: string; uuid: string; name: string; instances: CloudInstance[]; ownerId: string } | null } | 
        { key: "cloud.library.list", input: never, result: CloudLibrary[] } | 
        { key: "cloud.locations.list", input: never, result: CloudLocation[] } | 
        { key: "ephemeralFiles.getMediaData", input: string, result: ({ type: "Image" } & ImageMetadata) | ({ type: "Video" } & VideoMetadata) | ({ type: "Audio" } & AudioMetadata) | null } | 
        { key: "files.get", input: LibraryArgs<number>, result: { item: Reference<ObjectWithFilePaths2>; nodes: CacheNode[] } | null } | 
        { key: "files.getConvertableImageExtensions", input: never, result: string[] } | 
        { key: "files.getMediaData", input: LibraryArgs<number>, result: MediaMetadata } | 
        { key: "files.getPath", input: LibraryArgs<number>, result: string | null } | 
        { key: "invalidation.test-invalidate", input: never, result: number } | 
        { key: "jobs.isActive", input: LibraryArgs<null>, result: boolean } | 
        { key: "jobs.reports", input: LibraryArgs<null>, result: JobGroup[] } | 
        { key: "labels.count", input: LibraryArgs<null>, result: number } | 
        { key: "labels.get", input: LibraryArgs<number>, result: { id: number; name: string; date_created: string | null; date_modified: string | null } | null } | 
        { key: "labels.getForObject", input: LibraryArgs<number>, result: Label[] } | 
        { key: "labels.getWithObjects", input: LibraryArgs<number[]>, result: { [key in number]: { date_created: string; object: { id: number } }[] } } | 
        { key: "labels.list", input: LibraryArgs<null>, result: Label[] } | 
        { key: "labels.listWithThumbnails", input: LibraryArgs<string>, result: ExplorerItem[] } | 
        { key: "library.kindStatistics", input: LibraryArgs<null>, result: KindStatistics } | 
        { key: "library.list", input: never, result: NormalisedResults<LibraryConfigWrapped> } | 
        { key: "library.statistics", input: LibraryArgs<null>, result: StatisticsResponse } | 
        { key: "locations.get", input: LibraryArgs<number>, result: { item: Reference<Location>; nodes: CacheNode[] } | null } | 
        { key: "locations.getWithRules", input: LibraryArgs<number>, result: { item: Reference<LocationWithIndexerRule>; nodes: CacheNode[] } | null } | 
        { key: "locations.indexer_rules.get", input: LibraryArgs<number>, result: NormalisedResult<IndexerRule> } | 
        { key: "locations.indexer_rules.list", input: LibraryArgs<null>, result: NormalisedResults<IndexerRule> } | 
        { key: "locations.indexer_rules.listForLocation", input: LibraryArgs<number>, result: NormalisedResults<IndexerRule> } | 
        { key: "locations.list", input: LibraryArgs<null>, result: NormalisedResults<Location> } | 
        { key: "locations.systemLocations", input: never, result: SystemLocations } | 
        { key: "models.image_detection.list", input: never, result: string[] } | 
        { key: "nodeState", input: never, result: NodeState } | 
        { key: "nodes.listLocations", input: LibraryArgs<string | null>, result: ExplorerItem[] } | 
        { key: "notifications.dismiss", input: NotificationId, result: null } | 
        { key: "notifications.dismissAll", input: never, result: null } | 
        { key: "notifications.get", input: never, result: Notification[] } | 
        { key: "p2p.state", input: never, result: JsonValue } | 
        { key: "preferences.get", input: LibraryArgs<null>, result: LibraryPreferences } | 
        { key: "search.objects", input: LibraryArgs<ObjectSearchArgs>, result: SearchData<ExplorerItem> } | 
        { key: "search.objectsCount", input: LibraryArgs<{ filters?: SearchFilterArgs[] }>, result: number } | 
        { key: "search.paths", input: LibraryArgs<FilePathSearchArgs>, result: SearchData<ExplorerItem> } | 
        { key: "search.pathsCount", input: LibraryArgs<{ filters?: SearchFilterArgs[] }>, result: number } | 
        { key: "search.saved.get", input: LibraryArgs<number>, result: { id: number; pub_id: number[]; search: string | null; filters: string | null; name: string | null; icon: string | null; description: string | null; date_created: string | null; date_modified: string | null } | null } | 
        { key: "search.saved.list", input: LibraryArgs<null>, result: SavedSearch[] } | 
        { key: "sync.enabled", input: LibraryArgs<null>, result: boolean } | 
        { key: "sync.messages", input: LibraryArgs<null>, result: CRDTOperation[] } | 
        { key: "tags.get", input: LibraryArgs<number>, result: { item: Reference<Tag>; nodes: CacheNode[] } | null } | 
        { key: "tags.getForObject", input: LibraryArgs<number>, result: NormalisedResults<Tag> } | 
        { key: "tags.getWithObjects", input: LibraryArgs<number[]>, result: { [key in number]: ({ date_created: string | null; object: { id: number } })[] } } | 
        { key: "tags.list", input: LibraryArgs<null>, result: NormalisedResults<Tag> } | 
        { key: "volumes.list", input: never, result: NormalisedResults<Volume> },
    mutations: 
        { key: "api.sendFeedback", input: Feedback, result: null } | 
        { key: "auth.logout", input: never, result: null } | 
        { key: "backups.backup", input: LibraryArgs<null>, result: string } | 
        { key: "backups.delete", input: string, result: null } | 
        { key: "backups.restore", input: string, result: null } | 
        { key: "cloud.library.create", input: LibraryArgs<null>, result: null } | 
        { key: "cloud.library.join", input: string, result: LibraryConfigWrapped } | 
        { key: "cloud.library.sync", input: LibraryArgs<null>, result: null } | 
        { key: "cloud.locations.create", input: string, result: CloudLocation } | 
        { key: "cloud.locations.remove", input: string, result: CloudLocation } | 
        { key: "cloud.locations.testing", input: TestingParams, result: null } | 
        { key: "cloud.setApiOrigin", input: string, result: null } | 
        { key: "ephemeralFiles.copyFiles", input: LibraryArgs<EphemeralFileSystemOps>, result: null } | 
        { key: "ephemeralFiles.createFolder", input: LibraryArgs<CreateEphemeralFolderArgs>, result: string } | 
        { key: "ephemeralFiles.cutFiles", input: LibraryArgs<EphemeralFileSystemOps>, result: null } | 
        { key: "ephemeralFiles.deleteFiles", input: LibraryArgs<string[]>, result: null } | 
        { key: "ephemeralFiles.renameFile", input: LibraryArgs<EphemeralRenameFileArgs>, result: null } | 
        { key: "files.convertImage", input: LibraryArgs<ConvertImageArgs>, result: null } | 
        { key: "files.copyFiles", input: LibraryArgs<OldFileCopierJobInit>, result: null } | 
        { key: "files.createFolder", input: LibraryArgs<CreateFolderArgs>, result: string } | 
        { key: "files.cutFiles", input: LibraryArgs<OldFileCutterJobInit>, result: null } | 
        { key: "files.deleteFiles", input: LibraryArgs<OldFileDeleterJobInit>, result: null } | 
        { key: "files.eraseFiles", input: LibraryArgs<OldFileEraserJobInit>, result: null } | 
        { key: "files.removeAccessTime", input: LibraryArgs<number[]>, result: null } | 
        { key: "files.renameFile", input: LibraryArgs<RenameFileArgs>, result: null } | 
        { key: "files.setFavorite", input: LibraryArgs<SetFavoriteArgs>, result: null } | 
        { key: "files.setNote", input: LibraryArgs<SetNoteArgs>, result: null } | 
        { key: "files.updateAccessTime", input: LibraryArgs<number[]>, result: null } | 
        { key: "invalidation.test-invalidate-mutation", input: LibraryArgs<null>, result: null } | 
        { key: "jobs.cancel", input: LibraryArgs<string>, result: null } | 
        { key: "jobs.clear", input: LibraryArgs<string>, result: null } | 
        { key: "jobs.clearAll", input: LibraryArgs<null>, result: null } | 
        { key: "jobs.generateLabelsForLocation", input: LibraryArgs<GenerateLabelsForLocationArgs>, result: null } | 
        { key: "jobs.generateThumbsForLocation", input: LibraryArgs<GenerateThumbsForLocationArgs>, result: null } | 
        { key: "jobs.identifyUniqueFiles", input: LibraryArgs<IdentifyUniqueFilesArgs>, result: null } | 
        { key: "jobs.objectValidator", input: LibraryArgs<ObjectValidatorArgs>, result: null } | 
        { key: "jobs.pause", input: LibraryArgs<string>, result: null } | 
        { key: "jobs.resume", input: LibraryArgs<string>, result: null } | 
        { key: "labels.delete", input: LibraryArgs<number>, result: null } | 
        { key: "library.create", input: CreateLibraryArgs, result: NormalisedResult<LibraryConfigWrapped> } | 
        { key: "library.delete", input: string, result: null } | 
        { key: "library.edit", input: EditLibraryArgs, result: null } | 
        { key: "library.startActor", input: LibraryArgs<string>, result: null } | 
        { key: "library.stopActor", input: LibraryArgs<string>, result: null } | 
        { key: "locations.addLibrary", input: LibraryArgs<LocationCreateArgs>, result: number | null } | 
        { key: "locations.create", input: LibraryArgs<LocationCreateArgs>, result: number | null } | 
        { key: "locations.delete", input: LibraryArgs<number>, result: null } | 
        { key: "locations.fullRescan", input: LibraryArgs<FullRescanArgs>, result: null } | 
        { key: "locations.indexer_rules.create", input: LibraryArgs<IndexerRuleCreateArgs>, result: null } | 
        { key: "locations.indexer_rules.delete", input: LibraryArgs<number>, result: null } | 
        { key: "locations.relink", input: LibraryArgs<string>, result: number } | 
        { key: "locations.subPathRescan", input: LibraryArgs<RescanArgs>, result: null } | 
        { key: "locations.update", input: LibraryArgs<LocationUpdateArgs>, result: null } | 
        { key: "nodes.edit", input: ChangeNodeNameArgs, result: null } | 
        { key: "nodes.updateThumbnailerPreferences", input: UpdateThumbnailerPreferences, result: null } | 
        { key: "p2p.acceptSpacedrop", input: [string, string | null], result: null } | 
        { key: "p2p.cancelSpacedrop", input: string, result: null } | 
        { key: "p2p.debugConnect", input: RemoteIdentity, result: string } | 
        { key: "p2p.spacedrop", input: SpacedropArgs, result: string } | 
        { key: "preferences.update", input: LibraryArgs<LibraryPreferences>, result: null } | 
        { key: "search.saved.create", input: LibraryArgs<{ name: string; search?: string | null; filters?: string | null; description?: string | null; icon?: string | null }>, result: null } | 
        { key: "search.saved.delete", input: LibraryArgs<number>, result: null } | 
        { key: "search.saved.update", input: LibraryArgs<[number, Args]>, result: null } | 
        { key: "sync.enable", input: LibraryArgs<null>, result: null } | 
        { key: "tags.assign", input: LibraryArgs<{ targets: Target[]; tag_id: number; unassign: boolean }>, result: null } | 
        { key: "tags.create", input: LibraryArgs<TagCreateArgs>, result: Tag } | 
        { key: "tags.delete", input: LibraryArgs<number>, result: null } | 
        { key: "tags.update", input: LibraryArgs<TagUpdateArgs>, result: null } | 
        { key: "toggleFeatureFlag", input: BackendFeature, result: null },
    subscriptions: 
        { key: "auth.loginSession", input: never, result: Response } | 
        { key: "invalidation.listen", input: never, result: InvalidateOperationEvent[] } | 
        { key: "jobs.newThumbnail", input: LibraryArgs<null>, result: string[] } | 
        { key: "jobs.progress", input: LibraryArgs<null>, result: JobProgressEvent } | 
        { key: "library.actors", input: LibraryArgs<null>, result: { [key in string]: boolean } } | 
        { key: "locations.online", input: never, result: number[][] } | 
        { key: "locations.quickRescan", input: LibraryArgs<LightScanArgs>, result: null } | 
        { key: "notifications.listen", input: never, result: Notification } | 
        { key: "p2p.events", input: never, result: P2PEvent } | 
        { key: "search.ephemeralPaths", input: LibraryArgs<EphemeralPathSearchArgs>, result: EphemeralPathsResultItem } | 
        { key: "sync.newMessage", input: LibraryArgs<null>, result: null }
};

export type Args = { search?: string | null; filters?: string | null; name?: string | null; icon?: string | null; description?: string | null }

export type AudioMetadata = { duration: number | null; audio_codec: string | null }

/**
 * All of the feature flags provided by the core itself. The frontend has it's own set of feature flags!
 * 
 * If you want a variant of this to show up on the frontend it must be added to `backendFeatures` in `useFeatureFlag.tsx`
 */
export type BackendFeature = "filesOverP2P" | "cloudSync"

export type Backup = ({ id: string; timestamp: string; library_id: string; library_name: string }) & { path: string }

export type BuildInfo = { version: string; commit: string }

export type CRDTOperation = { instance: string; timestamp: number; id: string; model: string; record_id: JsonValue; data: CRDTOperationData }

export type CRDTOperationData = "c" | { u: { field: string; value: JsonValue } } | "d"

export type CacheNode = { __type: string; __id: string; "#node": any }

export type CameraData = { device_make: string | null; device_model: string | null; color_space: string | null; color_profile: ColorProfile | null; focal_length: number | null; shutter_speed: number | null; flash: Flash | null; orientation: Orientation; lens_make: string | null; lens_model: string | null; bit_depth: number | null; red_eye: boolean | null; zoom: number | null; iso: number | null; software: string | null; serial_number: string | null; lens_serial_number: string | null; contrast: number | null; saturation: number | null; sharpness: number | null; composite: Composite | null }

export type ChangeNodeNameArgs = { name: string | null; p2p_ipv4_port: Port | null; p2p_ipv6_port: Port | null; p2p_discovery: P2PDiscoveryState | null; image_labeler_version: string | null }

export type CloudInstance = { id: string; uuid: string; identity: RemoteIdentity; nodeId: string; nodeName: string; nodePlatform: number }

export type CloudLibrary = { id: string; uuid: string; name: string; instances: CloudInstance[]; ownerId: string }

export type CloudLocation = { id: string; name: string }

export type ColorProfile = "Normal" | "Custom" | "HDRNoOriginal" | "HDRWithOriginal" | "OriginalForHDR" | "Panorama" | "PortraitHDR" | "Portrait"

export type Composite = 
/**
 * The data is present, but we're unable to determine what they mean
 */
"Unknown" | 
/**
 * Not a composite image
 */
"False" | 
/**
 * A general composite image
 */
"General" | 
/**
 * The composite image was captured while shooting
 */
"Live"

export type ConvertImageArgs = { location_id: number; file_path_id: number; delete_src: boolean; desired_extension: ConvertibleExtension; quality_percentage: number | null }

export type ConvertibleExtension = "bmp" | "dib" | "ff" | "gif" | "ico" | "jpg" | "jpeg" | "png" | "pnm" | "qoi" | "tga" | "icb" | "vda" | "vst" | "tiff" | "tif" | "hif" | "heif" | "heifs" | "heic" | "heics" | "avif" | "avci" | "avcs" | "svg" | "svgz" | "pdf" | "webp"

export type CreateEphemeralFolderArgs = { path: string; name: string | null }

export type CreateFolderArgs = { location_id: number; sub_path: string | null; name: string | null }

export type CreateLibraryArgs = { name: LibraryName; default_locations: DefaultLocations | null }

export type CursorOrderItem<T> = { order: SortOrder; data: T }

export type DefaultLocations = { desktop: boolean; documents: boolean; downloads: boolean; pictures: boolean; music: boolean; videos: boolean }

export type DiskType = "SSD" | "HDD" | "Removable"

export type DoubleClickAction = "openFile" | "quickPreview"

export type EditLibraryArgs = { id: string; name: LibraryName | null; description: MaybeUndefined<string> }

export type EphemeralFileSystemOps = { sources: string[]; target_dir: string }

export type EphemeralPathOrder = { field: "name"; value: SortOrder } | { field: "sizeInBytes"; value: SortOrder } | { field: "dateCreated"; value: SortOrder } | { field: "dateModified"; value: SortOrder }

export type EphemeralPathSearchArgs = { path: string; withHiddenFiles: boolean; order?: EphemeralPathOrder | null }

export type EphemeralPathsResultItem = { entries: Reference<ExplorerItem>[]; errors: Error[]; nodes: CacheNode[] }

export type EphemeralRenameFileArgs = { kind: EphemeralRenameKind }

export type EphemeralRenameKind = { One: EphemeralRenameOne } | { Many: EphemeralRenameMany }

export type EphemeralRenameMany = { from_pattern: FromPattern; to_pattern: string; from_paths: string[] }

export type EphemeralRenameOne = { from_path: string; to: string }

export type Error = { code: ErrorCode; message: string }

/**
 * TODO
 */
export type ErrorCode = "BadRequest" | "Unauthorized" | "Forbidden" | "NotFound" | "Timeout" | "Conflict" | "PreconditionFailed" | "PayloadTooLarge" | "MethodNotSupported" | "ClientClosedRequest" | "InternalServerError"

export type ExplorerItem = { type: "Path"; thumbnail: string[] | null; item: FilePathWithObject } | { type: "Object"; thumbnail: string[] | null; item: ObjectWithFilePaths } | { type: "Location"; item: Location } | { type: "NonIndexedPath"; thumbnail: string[] | null; item: NonIndexedPathItem } | { type: "SpacedropPeer"; item: PeerMetadata } | { type: "Label"; thumbnails: string[][]; item: LabelWithObjects }

export type ExplorerLayout = "grid" | "list" | "media"

export type ExplorerSettings<TOrder> = { layoutMode: ExplorerLayout | null; gridItemSize: number | null; gridGap: number | null; mediaColumns: number | null; mediaAspectSquare: boolean | null; mediaViewWithDescendants: boolean | null; openOnDoubleClick: DoubleClickAction | null; showBytesInGridView: boolean | null; colVisibility: { [key in string]: boolean } | null; colSizes: { [key in string]: number } | null; order?: TOrder | null; showHiddenFiles?: boolean }

export type Feedback = { message: string; emoji: number }

export type FilePath = { id: number; pub_id: number[]; is_dir: boolean | null; cas_id: string | null; integrity_checksum: string | null; location_id: number | null; materialized_path: string | null; name: string | null; extension: string | null; hidden: boolean | null; size_in_bytes: string | null; size_in_bytes_bytes: number[] | null; inode: number[] | null; object_id: number | null; key_id: number | null; date_created: string | null; date_modified: string | null; date_indexed: string | null }

export type FilePathCursor = { isDir: boolean; variant: FilePathCursorVariant }

export type FilePathCursorVariant = "none" | { name: CursorOrderItem<string> } | { sizeInBytes: SortOrder } | { dateCreated: CursorOrderItem<string> } | { dateModified: CursorOrderItem<string> } | { dateIndexed: CursorOrderItem<string> } | { object: FilePathObjectCursor }

export type FilePathFilterArgs = { locations: InOrNotIn<number> } | { path: { location_id: number; path: string; include_descendants: boolean } } | { name: TextMatch } | { extension: InOrNotIn<string> } | { createdAt: Range<string> } | { modifiedAt: Range<string> } | { indexedAt: Range<string> } | { hidden: boolean }

export type FilePathObjectCursor = { dateAccessed: CursorOrderItem<string> } | { kind: CursorOrderItem<number> }

export type FilePathOrder = { field: "name"; value: SortOrder } | { field: "sizeInBytes"; value: SortOrder } | { field: "dateCreated"; value: SortOrder } | { field: "dateModified"; value: SortOrder } | { field: "dateIndexed"; value: SortOrder } | { field: "object"; value: ObjectOrder }

export type FilePathSearchArgs = { take?: number | null; orderAndPagination?: OrderAndPagination<number, FilePathOrder, FilePathCursor> | null; filters?: SearchFilterArgs[]; groupDirectories?: boolean }

export type FilePathWithObject = { id: number; pub_id: number[]; is_dir: boolean | null; cas_id: string | null; integrity_checksum: string | null; location_id: number | null; materialized_path: string | null; name: string | null; extension: string | null; hidden: boolean | null; size_in_bytes: string | null; size_in_bytes_bytes: number[] | null; inode: number[] | null; object_id: number | null; key_id: number | null; date_created: string | null; date_modified: string | null; date_indexed: string | null; object: { id: number; pub_id: number[]; kind: number | null; key_id: number | null; hidden: boolean | null; favorite: boolean | null; important: boolean | null; note: string | null; date_created: string | null; date_accessed: string | null } | null }

export type Flash = { 
/**
 * Specifies how flash was used (on, auto, off, forced, onvalid)
 * 
 * [`FlashMode::Unknown`] isn't a valid EXIF state, but it's included as the default,
 * just in case we're unable to correctly match it to a known (valid) state.
 * 
 * This type should only ever be evaluated if flash EXIF data is present, so having this as a non-option shouldn't be an issue.
 */
mode: FlashMode; 
/**
 * Did the flash actually fire?
 */
fired: boolean | null; 
/**
 * Did flash return to the camera? (Unsure of the meaning)
 */
returned: boolean | null; 
/**
 * Was red eye reduction used?
 */
red_eye_reduction: boolean | null }

export type FlashMode = 
/**
 * The data is present, but we're unable to determine what they mean
 */
"Unknown" | 
/**
 * FLash was on
 */
"On" | 
/**
 * Flash was off
 */
"Off" | 
/**
 * Flash was set to automatically fire in certain conditions
 */
"Auto" | 
/**
 * Flash was forcefully fired
 */
"Forced"

export type FromPattern = { pattern: string; replace_all: boolean }

export type FullRescanArgs = { location_id: number; reidentify_objects: boolean }

export type GenerateLabelsForLocationArgs = { id: number; path: string; regenerate?: boolean }

export type GenerateThumbsForLocationArgs = { id: number; path: string; regenerate?: boolean }

export type GetAll = { backups: Backup[]; directory: string }

export type HardwareModel = "Other" | "MacStudio" | "MacBookAir" | "MacBookPro" | "MacBook" | "MacMini" | "MacPro" | "IMac" | "IMacPro" | "IPad" | "IPhone" | "Simulator" | "Android"

export type IdentifyUniqueFilesArgs = { id: number; path: string }

export type ImageMetadata = { resolution: Resolution; date_taken: MediaDate | null; location: MediaLocation | null; camera_data: CameraData; artist: string | null; description: string | null; copyright: string | null; exif_version: string | null }

export type InOrNotIn<T> = { in: T[] } | { notIn: T[] }

export type IndexerRule = { id: number; pub_id: number[]; name: string | null; default: boolean | null; rules_per_kind: number[] | null; date_created: string | null; date_modified: string | null }

/**
 * `IndexerRuleCreateArgs` is the argument received from the client using rspc to create a new indexer rule.
 * Note that `rules` field is a vector of tuples of `RuleKind` and `parameters`.
 * 
 * In case of  `RuleKind::AcceptFilesByGlob` or `RuleKind::RejectFilesByGlob`, it will be a
 * vector of strings containing a glob patterns.
 * 
 * In case of `RuleKind::AcceptIfChildrenDirectoriesArePresent` or `RuleKind::RejectIfChildrenDirectoriesArePresent` the
 * `parameters` field must be a vector of strings containing the names of the directories.
 */
export type IndexerRuleCreateArgs = { name: string; dry_run: boolean; rules: ([RuleKind, string[]])[] }

export type InvalidateOperationEvent = { type: "single"; data: SingleInvalidateOperationEvent } | { type: "all" }

export type JobGroup = { id: string; action: string | null; status: JobStatus; created_at: string; jobs: JobReport[] }

export type JobProgressEvent = { id: string; library_id: string; task_count: number; completed_task_count: number; phase: string; message: string; estimated_completion: string }

export type JobReport = { id: string; name: string; action: string | null; data: number[] | null; metadata: { [key in string]: JsonValue } | null; errors_text: string[]; created_at: string | null; started_at: string | null; completed_at: string | null; parent_id: string | null; status: JobStatus; task_count: number; completed_task_count: number; phase: string; message: string; estimated_completion: string }

export type JobStatus = "Queued" | "Running" | "Completed" | "Canceled" | "Failed" | "Paused" | "CompletedWithErrors"

export type JsonValue = null | boolean | number | string | JsonValue[] | { [key in string]: JsonValue }

export type KindStatistic = { kind: number; name: string; count: number; total_bytes: string }

export type KindStatistics = { statistics: KindStatistic[] }

export type Label = { id: number; name: string; date_created: string | null; date_modified: string | null }

export type LabelWithObjects = { id: number; name: string; date_created: string | null; date_modified: string | null; label_objects: { object: { id: number; file_paths: FilePath[] } }[] }

/**
 * Can wrap a query argument to require it to contain a `library_id` and provide helpers for working with libraries.
 */
export type LibraryArgs<T> = { library_id: string; arg: T }

/**
 * LibraryConfig holds the configuration for a specific library. This is stored as a '{uuid}.sdlibrary' file.
 */
export type LibraryConfig = { 
/**
 * name is the display name of the library. This is used in the UI and is set by the user.
 */
name: LibraryName; 
/**
 * description is a user set description of the library. This is used in the UI and is set by the user.
 */
description: string | null; 
/**
 * id of the current instance so we know who this `.db` is. This can be looked up within the `Instance` table.
 */
instance_id: number; 
/**
 * cloud_id is the ID of the cloud library this library is linked to.
 * If this is set we can assume the library is synced with the Cloud.
 */
cloud_id?: string | null; generate_sync_operations?: boolean; version: LibraryConfigVersion }

export type LibraryConfigVersion = "V0" | "V1" | "V2" | "V3" | "V4" | "V5" | "V6" | "V7" | "V8" | "V9"

export type LibraryConfigWrapped = { uuid: string; instance_id: string; instance_public_key: RemoteIdentity; config: LibraryConfig }

export type LibraryName = string

export type LibraryPreferences = { location?: { [key in string]: LocationSettings } }

export type LightScanArgs = { location_id: number; sub_path: string }

export type Listener2 = { id: string; name: string; addrs: string[] }

export type Location = { id: number; pub_id: number[]; name: string | null; path: string | null; total_capacity: number | null; available_capacity: number | null; size_in_bytes: number[] | null; is_archived: boolean | null; generate_preview_media: boolean | null; sync_preview_media: boolean | null; hidden: boolean | null; date_created: string | null; instance_id: number | null }

/**
 * `LocationCreateArgs` is the argument received from the client using `rspc` to create a new location.
 * It has the actual path and a vector of indexer rules ids, to create many-to-many relationships
 * between the location and indexer rules.
 */
export type LocationCreateArgs = { path: string; dry_run: boolean; indexer_rules_ids: number[] }

export type LocationSettings = { explorer: ExplorerSettings<FilePathOrder> }

/**
 * `LocationUpdateArgs` is the argument received from the client using `rspc` to update a location.
 * It contains the id of the location to be updated, possible a name to change the current location's name
 * and a vector of indexer rules ids to add or remove from the location.
 * 
 * It is important to note that only the indexer rule ids in this vector will be used from now on.
 * Old rules that aren't in this vector will be purged.
 */
export type LocationUpdateArgs = { id: number; name: string | null; generate_preview_media: boolean | null; sync_preview_media: boolean | null; hidden: boolean | null; indexer_rules_ids: number[]; path: string | null }

export type LocationWithIndexerRule = { id: number; pub_id: number[]; name: string | null; path: string | null; total_capacity: number | null; available_capacity: number | null; size_in_bytes: number[] | null; is_archived: boolean | null; generate_preview_media: boolean | null; sync_preview_media: boolean | null; hidden: boolean | null; date_created: string | null; instance_id: number | null; indexer_rules: Reference<IndexerRule>[] }

export type MaybeUndefined<T> = null | T

export type MediaDataOrder = { field: "epochTime"; value: SortOrder }

/**
 * This can be either naive with no TZ (`YYYY-MM-DD HH-MM-SS`) or UTC (`YYYY-MM-DD HH-MM-SS ±HHMM`),
 * where `±HHMM` is the timezone data. It may be negative if West of the Prime Meridian, or positive if East.
 */
export type MediaDate = string

export type MediaLocation = { latitude: number; longitude: number; pluscode: PlusCode; altitude: number | null; direction: number | null }

export type MediaMetadata = ({ type: "Image" } & ImageMetadata) | ({ type: "Video" } & VideoMetadata) | ({ type: "Audio" } & AudioMetadata)

export type NodePreferences = { thumbnailer: ThumbnailerPreferences }

export type NodeState = ({ 
/**
 * id is a unique identifier for the current node. Each node has a public identifier (this one) and is given a local id for each library (done within the library code).
 */
id: string; 
/**
 * name is the display name of the current node. This is set by the user and is shown in the UI. // TODO: Length validation so it can fit in DNS record
 */
name: string; identity: RemoteIdentity; p2p_ipv4_port: Port; p2p_ipv6_port: Port; p2p_discovery: P2PDiscoveryState; features: BackendFeature[]; preferences: NodePreferences; image_labeler_version: string | null }) & { data_path: string; listeners: Listener2[]; device_model: string | null }

export type NonIndexedPathItem = { path: string; name: string; extension: string; kind: number; is_dir: boolean; date_created: string; date_modified: string; size_in_bytes_bytes: number[]; hidden: boolean }

/**
 * A type that can be used to return a group of `Reference<T>` and `CacheNode`'s
 * 
 * You don't need to use this, it's just a shortcut to avoid having to write out the full type every time.
 */
export type NormalisedResult<T> = { item: Reference<T>; nodes: CacheNode[] }

/**
 * A type that can be used to return a group of `Reference<T>` and `CacheNode`'s
 * 
 * You don't need to use this, it's just a shortcut to avoid having to write out the full type every time.
 */
export type NormalisedResults<T> = { items: Reference<T>[]; nodes: CacheNode[] }

/**
 * Represents a single notification.
 */
export type Notification = ({ type: "library"; id: [string, number] } | { type: "node"; id: number }) & { data: NotificationData; read: boolean; expires: string | null }

/**
 * Represents the data of a single notification.
 * This data is used by the frontend to properly display the notification.
 */
export type NotificationData = { title: string; content: string; kind: NotificationKind }

export type NotificationId = { type: "library"; id: [string, number] } | { type: "node"; id: number }

export type NotificationKind = "info" | "success" | "error" | "warning"

export type Object = { id: number; pub_id: number[]; kind: number | null; key_id: number | null; hidden: boolean | null; favorite: boolean | null; important: boolean | null; note: string | null; date_created: string | null; date_accessed: string | null }

export type ObjectCursor = "none" | { dateAccessed: CursorOrderItem<string> } | { kind: CursorOrderItem<number> }

export type ObjectFilterArgs = { favorite: boolean } | { hidden: ObjectHiddenFilter } | { kind: InOrNotIn<number> } | { tags: InOrNotIn<number> } | { labels: InOrNotIn<number> } | { dateAccessed: Range<string> }

export type ObjectHiddenFilter = "exclude" | "include"

export type ObjectOrder = { field: "dateAccessed"; value: SortOrder } | { field: "kind"; value: SortOrder } | { field: "mediaData"; value: MediaDataOrder }

export type ObjectSearchArgs = { take: number; orderAndPagination?: OrderAndPagination<number, ObjectOrder, ObjectCursor> | null; filters?: SearchFilterArgs[] }

export type ObjectValidatorArgs = { id: number; path: string }

export type ObjectWithFilePaths = { id: number; pub_id: number[]; kind: number | null; key_id: number | null; hidden: boolean | null; favorite: boolean | null; important: boolean | null; note: string | null; date_created: string | null; date_accessed: string | null; file_paths: FilePath[] }

export type ObjectWithFilePaths2 = { id: number; pub_id: number[]; kind: number | null; key_id: number | null; hidden: boolean | null; favorite: boolean | null; important: boolean | null; note: string | null; date_created: string | null; date_accessed: string | null; file_paths: Reference<FilePath>[] }

export type OldFileCopierJobInit = { source_location_id: number; target_location_id: number; sources_file_path_ids: number[]; target_location_relative_directory_path: string }

export type OldFileCutterJobInit = { source_location_id: number; target_location_id: number; sources_file_path_ids: number[]; target_location_relative_directory_path: string }

export type OldFileDeleterJobInit = { location_id: number; file_path_ids: number[] }

export type OldFileEraserJobInit = { location_id: number; file_path_ids: number[]; passes: string }

/**
 * Represents the operating system which the remote peer is running.
 * This is not used internally and predominantly is designed to be used for display purposes by the embedding application.
 */
export type OperatingSystem = "Windows" | "Linux" | "MacOS" | "Ios" | "Android" | { Other: string }

export type OrderAndPagination<TId, TOrder, TCursor> = { orderOnly: TOrder } | { offset: { offset: number; order: TOrder | null } } | { cursor: { id: TId; cursor: TCursor } }

export type Orientation = "Normal" | "CW90" | "CW180" | "CW270" | "MirroredVertical" | "MirroredHorizontal" | "MirroredHorizontalAnd90CW" | "MirroredHorizontalAnd270CW"

export type P2PDiscoveryState = "Everyone" | "ContactsOnly" | "Disabled"

/**
 * TODO: P2P event for the frontend
 */
export type P2PEvent = { type: "DiscoveredPeer"; identity: RemoteIdentity; metadata: PeerMetadata } | { type: "ExpiredPeer"; identity: RemoteIdentity } | { type: "ConnectedPeer"; identity: RemoteIdentity } | { type: "DisconnectedPeer"; identity: RemoteIdentity } | { type: "SpacedropRequest"; id: string; identity: RemoteIdentity; peer_name: string; files: string[] } | { type: "SpacedropProgress"; id: string; percent: number } | { type: "SpacedropTimedOut"; id: string } | { type: "SpacedropRejected"; id: string }

export type PeerMetadata = { name: string; operating_system: OperatingSystem | null; device_model: HardwareModel | null; version: string | null }

export type PlusCode = string

export type Port = null | number

export type Range<T> = { from: T } | { to: T }

/**
 * A reference to a `CacheNode`.
 * 
 * This does not contain the actual data, but instead a reference to it.
 * This allows the CacheNode's to be switched out and the query recomputed without any backend communication.
 * 
 * If you use a `Reference` in a query, you *must* ensure the corresponding `CacheNode` is also in the query.
 */
export type Reference<T> = { __type: string; __id: string; "#type": T }

export type RemoteIdentity = string

export type RenameFileArgs = { location_id: number; kind: RenameKind }

export type RenameKind = { One: RenameOne } | { Many: RenameMany }

export type RenameMany = { from_pattern: FromPattern; to_pattern: string; from_file_path_ids: number[] }

export type RenameOne = { from_file_path_id: number; to: string }

export type RescanArgs = { location_id: number; sub_path: string }

export type Resolution = { width: number; height: number }

export type Response = { Start: { user_code: string; verification_url: string; verification_url_complete: string } } | "Complete" | { Error: string }

export type RuleKind = "AcceptFilesByGlob" | "RejectFilesByGlob" | "AcceptIfChildrenDirectoriesArePresent" | "RejectIfChildrenDirectoriesArePresent"

export type SavedSearch = { id: number; pub_id: number[]; search: string | null; filters: string | null; name: string | null; icon: string | null; description: string | null; date_created: string | null; date_modified: string | null }

export type SearchData<T> = { cursor: number[] | null; items: Reference<T>[]; nodes: CacheNode[] }

export type SearchFilterArgs = { filePath: FilePathFilterArgs } | { object: ObjectFilterArgs }

export type SetFavoriteArgs = { id: number; favorite: boolean }

export type SetNoteArgs = { id: number; note: string | null }

export type SingleInvalidateOperationEvent = { 
/**
 * This fields are intentionally private.
 */
key: string; arg: JsonValue; result: JsonValue | null }

export type SortOrder = "Asc" | "Desc"

export type SpacedropArgs = { identity: RemoteIdentity; file_path: string[] }

export type Statistics = { id: number; date_captured: string; total_object_count: number; library_db_size: string; total_bytes_used: string; total_bytes_capacity: string; total_unique_bytes: string; total_bytes_free: string; preview_media_bytes: string }

export type StatisticsResponse = { statistics: Statistics | null }

export type SystemLocations = { desktop: string | null; documents: string | null; downloads: string | null; pictures: string | null; music: string | null; videos: string | null }

export type Tag = { id: number; pub_id: number[]; name: string | null; color: string | null; is_hidden: boolean | null; date_created: string | null; date_modified: string | null }

export type TagCreateArgs = { name: string; color: string }

export type TagUpdateArgs = { id: number; name: string | null; color: string | null }

export type Target = { Object: number } | { FilePath: number }

export type TestingParams = { id: string; path: string }

export type TextMatch = { contains: string } | { startsWith: string } | { endsWith: string } | { equals: string }

export type ThumbnailerPreferences = { background_processing_percentage: number }

export type UpdateThumbnailerPreferences = { background_processing_percentage: number }

export type VideoMetadata = { duration: number | null; video_codec: string | null; audio_codec: string | null }

export type Volume = { name: string; mount_points: string[]; total_capacity: string; available_capacity: string; disk_type: DiskType; file_system: string | null; is_root_filesystem: boolean }
