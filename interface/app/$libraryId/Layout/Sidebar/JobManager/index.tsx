import { Check, Trash, X } from '@phosphor-icons/react';
import { useQueryClient } from '@tanstack/react-query';
import dayjs from 'dayjs';
import { useState } from 'react';
import {
	JobGroup as IJobGroup,
	useJobProgress,
	useLibraryMutation,
	useLibraryQuery
} from '@sd/client';
import { Button, PopoverClose, toast, Tooltip } from '@sd/ui';
import { useIsDark, useLocale } from '~/hooks';

import IsRunningJob from './IsRunningJob';
import JobGroup from './JobGroup';

function sortJobData(jobs: IJobGroup[]) {
	const runningJobs: IJobGroup[] = [];
	const otherJobs: IJobGroup[] = [];

	jobs.forEach((job) => {
		if (job.status === 'Running' || job.jobs.find((job) => job.status === 'Running')) {
			runningJobs.push(job);
		} else {
			otherJobs.push(job);
		}
	});

	const sortByCreatedAt = (a: IJobGroup, b: IJobGroup) => {
		const aDate = dayjs(a.created_at);
		const bDate = dayjs(b.created_at);
		if (aDate.isBefore(bDate)) {
			return 1;
		} else if (bDate.isBefore(aDate)) {
			return -1;
		}
		return 0;
	};

	runningJobs.sort(sortByCreatedAt);
	otherJobs.sort(sortByCreatedAt);

	return [...runningJobs, ...otherJobs];
}

export function JobManager() {
	const queryClient = useQueryClient();
	const [toggleConfirmation, setToggleConfirmation] = useState(false);

	const jobGroups = useLibraryQuery(['jobs.reports']);

	const progress = useJobProgress(jobGroups.data);

	const isDark = useIsDark();

	const { t } = useLocale();

	const clearAllJobs = useLibraryMutation(['jobs.clearAll'], {
		onError: () => {
			toast.error({
				title: t('error'),
				body: t('failed_to_clear_all_jobs')
			});
		},
		onSuccess: () => {
			queryClient.invalidateQueries(['jobs.reports ']);
			setToggleConfirmation((t) => !t);
			toast.success({
				title: t('success'),
				body: t('all_jobs_have_been_cleared')
			});
		}
	});

	const clearAllJobsHandler = () => {
		clearAllJobs.mutate(null);
	};

	return (
		<div className="h-full overflow-hidden pb-10">
			<div className="z-20 flex h-9 w-full items-center rounded-t-md border-b border-app-line/50 bg-app-button/30 px-2">
				<span className=" ml-1.5 font-medium">{t('recent_jobs')}</span>
				<div className="grow" />
				{toggleConfirmation ? (
					<div className="flex h-[85%] w-fit items-center justify-center gap-2 rounded-md border border-app-line bg-app/40 px-2">
						<p className="text-[10px]">{t('are_you_sure')}</p>
						<PopoverClose asChild>
							<Check
								onClick={clearAllJobsHandler}
								className="size-3 transition-opacity duration-300 hover:opacity-70"
								color={isDark ? 'white' : 'black'}
							/>
						</PopoverClose>
						<X
							className="size-3 transition-opacity hover:opacity-70"
							onClick={() => setToggleConfirmation((t) => !t)}
						/>
					</div>
				) : (
					<Button
						className="opacity-70"
						onClick={() => setToggleConfirmation((t) => !t)}
						size="icon"
					>
						<Tooltip label={t('clear_finished_jobs')}>
							<Trash className="size-4" />
						</Tooltip>
					</Button>
				)}
				<PopoverClose asChild>
					<Button className="opacity-70" size="icon">
						<Tooltip label={t('close')}>
							<X className="size-4" />
						</Tooltip>
					</Button>
				</PopoverClose>
			</div>
			<div className="custom-scroll job-manager-scroll h-full overflow-x-hidden">
				<div className="h-full border-r border-app-line/50">
					{jobGroups.data &&
						(jobGroups.data.length === 0 ? (
							<div className="flex h-32 items-center justify-center text-sidebar-inkDull">
								{t('no_jobs')}
							</div>
						) : (
							sortJobData(jobGroups.data).map((group) => (
								<JobGroup key={group.id} group={group} progress={progress} />
							))
						))}
				</div>
			</div>
		</div>
	);
}

export { IsRunningJob };
