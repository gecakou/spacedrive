import { AlphaRSPCError, Link, RspcRequest } from '@rspc/client/v2';
import { NativeEventEmitter, NativeModules } from 'react-native';

const { SDCore } = NativeModules;
const eventEmitter = new NativeEventEmitter(NativeModules.SDCore);

/**
 * Link for the custom React Native rspc backend
 */
export function reactNativeLink(): Link {
	const activeMap = new Map<
		string,
		{
			resolve: (result: any) => void;
			reject: (error: Error | AlphaRSPCError) => void;
		}
	>();

	const handleIncoming = (event: any) => {
		const { id, result } = event;
		if (activeMap.has(id)) {
			if (result.type === 'event') {
				activeMap.get(id)?.resolve(result.data);
			} else if (result.type === 'response') {
				activeMap.get(id)?.resolve(result.data);
				activeMap.delete(id);
			} else if (result.type === 'error') {
				const { message, code } = result.data;
				activeMap.get(id)?.reject(new AlphaRSPCError(code, message));
				activeMap.delete(id);
			} else {
				console.error(`rspc: received event of unknown type '${result.type}'`);
			}
		} else {
			console.error(`rspc: received event for unknown id '${id}'`);
		}
	};

	eventEmitter.addListener('SDCoreEvent', (event) => {
		handleIncoming(JSON.parse(event));
	});

	const batch: RspcRequest[] = [];
	let batchQueued = false;
	const queueBatch = () => {
		if (!batchQueued) {
			batchQueued = true;
			setTimeout(() => {
				const currentBatch = [...batch];
				(async () => {
					const data = JSON.parse(await SDCore.sd_core_msg(currentBatch));
					if (Array.isArray(data)) {
						for (const payload of data) {
							handleIncoming(payload);
						}
					} else {
						handleIncoming(data);
					}
				})();

				batch.splice(0, batch.length);
				batchQueued = false;
			});
		}
	};

	return ({ op }) => {
		let finished = false;
		return {
			exec: async (resolve, reject) => {
				activeMap.set(op.id, {
					resolve,
					reject
				});
				// @ts-expect-error // TODO: Fix this
				batch.push({
					id: op.id,
					method: op.type,
					params: {
						path: op.path,
						input: op.input
					}
				});
				queueBatch();
			},
			abort() {
				if (finished) return;
				finished = true;

				activeMap.delete(op.id);

				batch.push({
					jsonrpc: '2.0',
					id: op.id,
					method: 'subscriptionStop',
					params: null
				});
				queueBatch();
			}
		};
	};
}
