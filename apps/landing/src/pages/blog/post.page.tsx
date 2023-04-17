import Prism from 'prismjs';
import 'prismjs/components/prism-rust';
import { useEffect } from 'react';
import { Helmet } from 'react-helmet';
import '../../atom-one.css';
import { BlogTag } from '../../components/BlogTag';
import { BlogPost } from './api';

function Page({ post }: { post: BlogPost }) {
	useEffect(() => {
		Prism.highlightAll();
	}, []);

	const description =
		post?.excerpt?.length || 0 > 160 ? post?.excerpt?.substring(0, 160) + '...' : post?.excerpt;

	const featured_image =
		post?.image ||
		'https://raw.githubusercontent.com/spacedriveapp/.github/main/profile/spacedrive_icon.png';

	return (
		<>
			<Helmet>
				<title>{post?.title} - Spacedrive Blog</title>
				<meta name="description" content={description} />
				<meta property="og:title" content={post?.title} />
				<meta property="og:description" content={description} />
				<meta property="og:image" content={featured_image} />
				<meta content="summary_large_image" name="twitter:card" />
				<meta name="author" content={post?.author || 'Spacedrive Technology Inc.'} />
			</Helmet>
			<div className="lg:prose-xs prose dark:prose-invert container m-auto mb-20 max-w-4xl p-4 pt-14">
				<>
					<figure>
						{/* <figcaption dangerouslySetInnerHTML={{ __html: post.imageCaption as any }}></figcaption> */}
						<img src={`/${featured_image}`} alt="" className="mt-8 rounded-xl" />
					</figure>
					<section className="-mx-8 flex flex-wrap gap-4 rounded-xl px-8">
						<div className="grow">
							<h1 className="m-0 text-2xl leading-snug sm:text-4xl sm:leading-normal">
								{post?.title}
							</h1>
							<p className="m-0 mt-2">
								by <b>{post?.author}</b> &middot;{' '}
								{new Date(post?.date ?? '').toLocaleDateString()}
							</p>
						</div>
						<div className="flex flex-wrap gap-2">
							{post.tags?.map((tag) => (
								<BlogTag key={tag} name={tag} />
							))}
						</div>
					</section>
					<article
						id="content"
						className="text-lg"
						dangerouslySetInnerHTML={{
							__html: post.html?.replaceAll(
								'<a href=',
								`<a target="_blank" rel="noreferrer" href=`
							) as string
						}}
					></article>
				</>
			</div>
		</>
	);
}

export { Page };
