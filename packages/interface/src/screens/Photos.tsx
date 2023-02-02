export default function PhotosScreen() {
	return (
		<div className="custom-scroll page-scroll app-background flex h-screen w-full flex-col p-5">
			<div className="flex flex-col space-y-5 pb-7">
				<p className="border-app-line bg-app-box mb-3 rounded-md border px-5 py-3 text-sm shadow-sm ">
					<b>Note: </b>This is a pre-alpha build of Spacedrive, many features are yet to be
					functional.
				</p>
				{/* <Spline
					style={{ height: 500 }}
					height={500}
					className="rounded-md shadow-sm pointer-events-auto"
					scene="https://prod.spline.design/KUmO4nOh8IizEiCx/scene.splinecode"
				/> */}
			</div>
		</div>
	);
}
