import { useQueryClient } from '@tanstack/react-query';
import dayjs from 'dayjs';
import { Info, Question } from 'phosphor-react';
import { memo, useCallback, useEffect, useState } from 'react';
import {
	JobProgressEvent,
	JobReport,
	useLibraryMutation,
	useLibrarySubscription
} from '@sd/client';
import { ProgressBar } from '@sd/ui';
import { showAlertDialog } from '~/components';
import JobContainer from './JobContainer';
import useJobInfo from './useJobInfo';

interface JobProps {
	job: JobReport;
	className?: string;
	isChild?: boolean;
}

function Job({ job, className, isChild }: JobProps) {
	const queryClient = useQueryClient();

	const [realtimeUpdate, setRealtimeUpdate] = useState<JobProgressEvent | null>(null);

	useLibrarySubscription(['jobs.progress', job.id], {
		onData: setRealtimeUpdate
	});

	const niceData = useJobInfo(job, realtimeUpdate)[job.name] || {
		name: job.name,
		icon: Question,
		textItems: [[{ text: job.status.replace(/([A-Z])/g, ' $1').trim() }]]
	};
	const isRunning = job.status === 'Running';
	const isPaused = job.status === 'Paused';

	const task_count = realtimeUpdate?.task_count || job.task_count;
	const completed_task_count = realtimeUpdate?.completed_task_count || job.completed_task_count;

	// clear stale realtime state when job is done
	useEffect(() => {
		if (isRunning) setRealtimeUpdate(null);
	}, [isRunning]);

	// dayjs from seconds to time
	// const timeText = isRunning ? formatEstimatedRemainingTime(job.estimated_completion) : undefined;

	// I don't like sending TSX as a prop due to lack of hot-reload, but it's the only way to get the error log to show up
	if (job.status === 'CompletedWithErrors') {
		const JobError = (
			<pre className="custom-scroll inspector-scroll max-h-[300px] rounded border border-app-darkBox bg-app-darkBox/80 p-3">
				{job.errors_text.map((error, i) => (
					<p
						className="mb-1 w-full overflow-auto whitespace-normal break-words text-sm"
						key={i}
					>
						{error.trim()}
					</p>
				))}
			</pre>
		);
		niceData.textItems?.push([
			{
				text: 'Completed with errors',
				icon: Info,
				onClick: () => {
					showAlertDialog({
						title: 'Error',
						description:
							'The job completed with errors. Please see the error log below for more information. If you need help, please contact support and provide this error.',
						children: JobError
					});
				}
			}
		]);
	}

	return (
		<JobContainer
			className={className}
			name={niceData.name}
			circleIcon={niceData.icon}
			textItems={
				['Queued'].includes(job.status) ? [[{ text: job.status }]] : niceData.textItems
			}
			// textItems={[[{ text: job.status }, { text: job.id, }]]}
			isChild={isChild}
		>
			{isRunning ||
				(isPaused && (
					<div className="my-1 ml-1.5 w-[335px]">
						<ProgressBar
							pending={task_count == 0}
							value={completed_task_count}
							total={task_count}
						/>
					</div>
				))}
		</JobContainer>
	);
}

export default memo(Job);

function formatEstimatedRemainingTime(end_date: string) {
	const duration = dayjs.duration(new Date(end_date).getTime() - Date.now());

	if (duration.hours() > 0) {
		return `${duration.hours()} hour${duration.hours() > 1 ? 's' : ''} remaining`;
	} else if (duration.minutes() > 0) {
		return `${duration.minutes()} minute${duration.minutes() > 1 ? 's' : ''} remaining`;
	} else {
		return `${duration.seconds()} second${duration.seconds() > 1 ? 's' : ''} remaining`;
	}
}
