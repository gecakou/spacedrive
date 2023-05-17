import clsx from 'clsx';
import { Dispatch, SetStateAction } from 'react';
import { IndexerRule } from '@sd/client';
import { IndexerRuleIdFieldType } from '.';
import { InfoPill } from '~/app/$libraryId/Explorer/Inspector';

interface RuleButtonProps<T extends IndexerRuleIdFieldType> {
	rule: IndexerRule;
	field?: T;
	setRuleSelected: Dispatch<SetStateAction<IndexerRule | undefined>>;
	ruleSelected: IndexerRule | undefined
}

function RuleButton<T extends IndexerRuleIdFieldType>({
	rule,
	field,
	ruleSelected,
	setRuleSelected
}: RuleButtonProps<T>) {
	const value = field?.value ?? [];
	const ruleEnabled = value.includes(rule.id);

	return (
		<div className={clsx(ruleSelected?.id === rule.id ? 'bg-app-darkBox' : 'bg-app-input', !rule.default && 'cursor-pointer',
		`w-[100px] border border-app-line min-w-[150px] relative flex justify-between gap-2 rounded-md py-2`)}
		 onClick={() => {
				if (rule.default) return;
			    ruleSelected?.id === rule.id ? setRuleSelected(undefined) :
				setRuleSelected(rule)
		}}>
				<div className='w-full'>
				<p className='mb-2 text-sm text-center'>{rule.name}</p>
					<div className='flex flex-wrap justify-center gap-2'>
					<InfoPill
					onClick={field && (() =>
						field.onChange(
							ruleEnabled
								? value.filter((v) => v !== rule.id)
								: Array.from(new Set([...value, rule.id]))
						))}
					className={clsx('hover:brightness-125', ruleEnabled ? '!text-green-500' : 'text-red-500')}>
						{ruleEnabled ? 'Enabled' : 'Disabled'}
					</InfoPill>
					{rule.default &&  (
						<InfoPill className='text-ink-faint'>
						System
					</InfoPill>
					)}
					</div>
				</div>
		</div>
	);
}

export default RuleButton;
