import dayjs from 'dayjs';
import {
	Copy,
	Heart,
	Icon,
	Info,
	LockSimple,
	LockSimpleOpen,
	Package,
	Pencil,
	Share,
	TrashSimple
} from 'phosphor-react-native';
import { PropsWithChildren, useRef } from 'react';
import { Alert, Pressable, Text, View, ViewStyle } from 'react-native';
import { ObjectKind, formatBytes, isObject, isPath, useLibraryQuery } from '@sd/client';
import FileThumb from '~/components/explorer/FileThumb';
import FavoriteButton from '~/components/explorer/actions/FavoriteButton';
import { Modal, ModalRef } from '~/components/layout/Modal';
import { InfoPill, PlaceholderPill } from '~/components/primitive/InfoPill';
import tw, { twStyle } from '~/lib/tailwind';
import { useActionsModalStore } from '~/stores/modalStore';
import FileInfoModal from './FileInfoModal';

type ActionsContainerProps = PropsWithChildren<{
	style?: ViewStyle;
}>;

const ActionsContainer = ({ children, style }: ActionsContainerProps) => (
	<View style={twStyle('bg-app-box rounded-lg py-3.5', style)}>{children}</View>
);

type ActionsItemProps = {
	title: string;
	icon: Icon;
	onPress?: () => void;
	isDanger?: boolean;
};

const ActionsItem = ({ icon, onPress, title, isDanger = false }: ActionsItemProps) => {
	const Icon = icon;
	return (
		<Pressable onPress={onPress} style={tw`flex flex-row items-center justify-between px-4`}>
			<Text
				style={twStyle(
					'text-base leading-none font-medium',
					isDanger ? 'text-red-600' : 'text-ink'
				)}
			>
				{title}
			</Text>
			<Icon color={isDanger ? 'red' : 'white'} size={22} />
		</Pressable>
	);
};

const ActionDivider = () => <View style={tw`my-3.5 h-[0.5px] bg-app-line/80`} />;

export const ActionsModal = () => {
	const fileInfoRef = useRef<ModalRef>(null);

	const { modalRef, data } = useActionsModalStore();

	const item = data?.item;

	const objectData = data ? (isObject(data) ? data.item : data.item.object) : null;
	const isDir = data && isPath(data) ? data.item.is_dir : false;

	const tagsQuery = useLibraryQuery(['tags.getForObject', objectData?.id], {
		enabled: Boolean(objectData)
	});

	return (
		<>
			<Modal ref={modalRef} snapPoints={['60%', '90%']}>
				{data && (
					<View style={tw`flex-1 px-4`}>
						<View style={tw`flex flex-row items-center`}>
							{/* Thumbnail/Icon */}
							<Pressable onPress={() => fileInfoRef.current.present()}>
								<FileThumb data={data} size={1} />
							</Pressable>
							<View style={tw`ml-2 flex-1`}>
								{/* Name + Extension */}
								<Text style={tw`text-base font-bold text-gray-200`} numberOfLines={1}>
									{item.name}
									{item.extension && `.${item.extension}`}
								</Text>
								<View style={tw`flex flex-row`}>
									<Text style={tw`text-ink-faint text-xs`}>
										{formatBytes(Number(objectData?.size_in_bytes || 0))},
									</Text>
									<Text style={tw`text-ink-faint text-xs`}>
										{' '}
										{dayjs(item.date_created).format('MMM Do YYYY')}
									</Text>
								</View>
								{/* Info pills w/ tags */}
								<View style={tw`flex flex-row flex-wrap mt-1`}>
									{/* Kind */}
									<InfoPill
										containerStyle={tw`mr-1`}
										text={isDir ? 'Folder' : ObjectKind[objectData?.kind || 0]}
									/>
									{/* Extension */}
									{item.extension && <InfoPill text={item.extension} containerStyle={tw`mr-1`} />}
									{/* TODO: What happens if I have too many? */}
									{tagsQuery.data?.map((tag) => (
										<InfoPill
											key={tag.id}
											text={tag.name}
											containerStyle={tw.style('mr-1', { backgroundColor: tag.color + 'CC' })}
											textStyle={tw`text-white`}
										/>
									))}
									<Pressable onPress={() => Alert.alert('TODO')}>
										<PlaceholderPill text={'Add Tag'} />
									</Pressable>
								</View>
							</View>
							<FavoriteButton style={tw`mr-4`} data={objectData} />
						</View>
						<View style={tw`my-3`} />
						{/* Actions */}
						<ActionsContainer>
							<ActionsItem
								icon={Info}
								title="Show Info"
								onPress={() => fileInfoRef.current.present()}
							/>
						</ActionsContainer>
						<ActionsContainer style={tw`mt-2`}>
							<ActionsItem icon={Pencil} title="Rename" />
							<ActionDivider />
							<ActionsItem icon={Copy} title="Duplicate" />
							<ActionDivider />
							<ActionsItem icon={Share} title="Share" />
						</ActionsContainer>
						<ActionsContainer style={tw`mt-2`}>
							<ActionsItem icon={LockSimple} title="Encrypt" />
							<ActionDivider />
							<ActionsItem icon={LockSimpleOpen} title="Decrypt" />
							<ActionDivider />
							<ActionsItem icon={Package} title="Compress" />
							<ActionDivider />
							<ActionsItem icon={TrashSimple} title="Delete" isDanger />
						</ActionsContainer>
					</View>
				)}
			</Modal>
			<FileInfoModal ref={fileInfoRef} data={data} />
		</>
	);
};
