import { AppLogo } from '@sd/assets/images';
import {
	Discord,
	Github,
	Instagram,
	Opencollective,
	Twitch,
	Twitter
} from '@sd/assets/svgs/brands';
import Image from 'next/image';
import Link from 'next/link';
import { PropsWithChildren } from 'react';

function FooterLink(props: PropsWithChildren<{ link: string; blank?: boolean }>) {
	return (
		<Link
			href={props.link}
			target={props.blank ? '_blank' : ''}
			className="text-gray-300 duration-300 hover:text-white hover:opacity-50"
			rel="noreferrer"
		>
			{props.children}
		</Link>
	);
}

export function Footer() {
	return (
		<footer id="footer" className="z-50 w-screen border-t border-gray-550 pt-3 backdrop-blur">
			<div className="min-h-64 m-auto grid max-w-[100rem] grid-cols-2 gap-6 p-8 pb-20 pt-10 text-white sm:grid-cols-2 lg:grid-cols-6">
				<div className="col-span-2">
					<Image alt="Spacedrive logo" src={AppLogo} className="mb-5 h-10 w-10" />

					<h3 className="mb-1 text-xl font-bold">Spacedrive</h3>
					<p className="text-sm text-gray-350 opacity-50">
						&copy; Copyright {new Date().getFullYear()} Spacedrive Technology Inc.
					</p>
					<div className="mb-10 mt-12 flex flex-row space-x-3">
						<FooterLink link="https://twitter.com/spacedriveapp">
							<Twitter className="h-6 w-6" />
						</FooterLink>
						<FooterLink link="https://discord.gg/gTaF2Z44f5">
							<Discord className="h-6 w-6" />
						</FooterLink>
						<FooterLink link="https://instagram.com/spacedriveapp">
							<Instagram className="h-6 w-6" />
						</FooterLink>
						<FooterLink link="https://github.com/spacedriveapp">
							<Github className="h-6 w-6" />
						</FooterLink>
						<FooterLink link="https://opencollective.com/spacedrive">
							<Opencollective className="h-6 w-6" />
						</FooterLink>
						<FooterLink link="https://twitch.tv/jamiepinelive">
							<Twitch className="h-6 w-6" />
						</FooterLink>
					</div>
				</div>

				<div className="col-span-1 flex flex-col space-y-2">
					<h3 className="mb-1 text-xs font-bold uppercase ">About</h3>

					<FooterLink link="/team">Team</FooterLink>
					<FooterLink link="/docs/product/resources/faq">FAQ</FooterLink>
					<FooterLink link="/careers">Careers</FooterLink>
					<FooterLink link="/docs/changelog/beta/0.1.0">Changelog</FooterLink>
					<FooterLink link="/blog">Blog</FooterLink>
				</div>
				<div className="pointer-events-none col-span-1 flex flex-col space-y-2">
					<h3 className="mb-1 text-xs font-bold uppercase">Downloads</h3>
					<div className="col-span-1 flex flex-col space-y-2 opacity-50">
						<FooterLink link="#">macOS</FooterLink>
						<FooterLink link="#">Windows</FooterLink>
						<FooterLink link="#">Linux</FooterLink>
						<FooterLink link="#">Android</FooterLink>
						<FooterLink link="#">iOS</FooterLink>
					</div>
				</div>
				<div className="col-span-1 flex flex-col space-y-2">
					<h3 className="mb-1 text-xs font-bold uppercase ">Developers</h3>
					<FooterLink link="/docs/product/getting-started/introduction">
						Documentation
					</FooterLink>
					<FooterLink
						blank
						link="https://github.com/spacedriveapp/spacedrive/blob/main/CONTRIBUTING.md"
					>
						Contribute
					</FooterLink>
					<div className="pointer-events-none opacity-50">
						<FooterLink link="#">Extensions</FooterLink>
					</div>
					<div className="pointer-events-none opacity-50">
						<FooterLink link="#">Self Host</FooterLink>
					</div>
				</div>
				<div className="col-span-1 flex flex-col space-y-2">
					<h3 className="mb-1 text-xs font-bold uppercase ">Org</h3>
					<FooterLink blank link="https://opencollective.com/spacedrive">
						Open Collective
					</FooterLink>
					<FooterLink
						blank
						link="https://github.com/spacedriveapp/spacedrive/blob/main/LICENSE"
					>
						License
					</FooterLink>
					<div>
						<FooterLink link="/docs/company/legal/privacy">Privacy</FooterLink>
					</div>
					<div>
						<FooterLink link="/docs/company/legal/terms">Terms</FooterLink>
					</div>
				</div>
			</div>
		</footer>
	);
}
