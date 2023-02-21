import clsx from 'clsx';
import { DeviceMobile, HardDrives, Icon, Laptop, PhoneX, User } from 'phosphor-react';
import { tw } from '@sd/ui';
import { OperatingSystem } from '../util/Platform';
import classes from './Spacedrop.module.scss';
import { ScreenContainer } from './_Layout';

// TODO: move this to UI, copied from Inspector
const Pill = tw.span`mt-1 inline border border-transparent px-0.5 text-[9px] font-medium shadow shadow-app-shade/5 bg-app-selected rounded text-ink-dull`;

type DropItemProps = {
	// TODO: remove optionals when dummy data is removed (except for icon)
	name?: string;
	connectionType?: 'lan' | 'bluetooth' | 'usb' | 'spacetunnel' | 'p2p';
	receivingNodeOsType?: Omit<OperatingSystem, 'unknown'>;
} & ({ image: string } | { icon?: Icon });

function DropItem(props: DropItemProps) {
	let icon;
	if ('image' in props) {
		icon = <img className="rounded-full" src={props.image} alt={props.name} />;
	} else {
		const Icon = props.icon || User;
		icon = <Icon className={clsx('w-8 h-8 m-3', !props.name && 'opacity-20')} />;
	}

	return (
		<div
			className={clsx(classes.honeycombItem, 'overflow-hidden bg-app-box/20 hover:bg-app-box/50')}
		>
			<div className="flex flex-col items-center justify-center w-full h-full">
				<div className="rounded-full w-14 h-14 bg-app-button">{icon}</div>
				{props.name && <span className="mt-1 text-xs font-medium">{props.name}</span>}
				<div className="flex flex-row space-x-1">
					{props.receivingNodeOsType && <Pill>{props.receivingNodeOsType}</Pill>}
					{props.connectionType && (
						<Pill
							className={clsx(
								'!text-white uppercase',
								props.connectionType === 'lan' && 'bg-green-500',
								props.connectionType === 'p2p' && 'bg-blue-500'
							)}
						>
							{props.connectionType}
						</Pill>
					)}
				</div>
			</div>
		</div>
	);
}

export default function SpacedropScreen() {
	return (
		<ScreenContainer className={classes.honeycombOuter}>
			<div className={classes.honeycombContainer}>
				<DropItem
					name="Jamie's MacBook Pro"
					receivingNodeOsType="macOs"
					connectionType="lan"
					icon={Laptop}
				/>
				<DropItem
					name="Jamie's iPhone"
					receivingNodeOsType="iOS"
					connectionType="lan"
					icon={DeviceMobile}
				/>
				<DropItem
					name="Titan NAS"
					receivingNodeOsType="linux"
					connectionType="p2p"
					icon={HardDrives}
				/>
				<DropItem
					name="Jamie's iPad"
					receivingNodeOsType="iOS"
					connectionType="lan"
					icon={DeviceMobile}
				/>
				<DropItem name="maxichrome" image="https://github.com/maxichrome.png" />
				<DropItem name="Brendan Alan" image="https://github.com/brendonovich.png" />
				<DropItem name="Oscar Beaumont" image="https://github.com/oscartbeaumont.png" />
				<DropItem name="Polar" image="https://github.com/polargh.png" />
				<DropItem name="Andrew Haskell" image="https://github.com/andrewtechx.png" />
				<DropItem />
				<DropItem />
				<DropItem />
				<DropItem />
				<DropItem />
				<DropItem />
				<DropItem />
				<DropItem />
				<DropItem />
				<DropItem />
				<DropItem />
				<DropItem />
				<DropItem />
				<DropItem />
				<DropItem />
				<DropItem />
			</div>
		</ScreenContainer>
	);
}
