import * as fs from 'node:fs/promises';
import * as path from 'node:path';

// https://stackoverflow.com/q/3651860#answer-67151923
const REF_REGEX = /ref:\s+refs\/heads\/(?<branch>[^\s\x00-\x1F\:\?\[\\\^\~]+)/;

/**
 * @param {string} repoPath
 * @returns {Promise<string[]>}
 */
export async function getGitBranches(repoPath) {
	const branches = ['main', 'master'];

	let head;
	try {
		head = await fs.readFile(path.join(repoPath, '.git', 'HEAD'), { encoding: 'utf8' });
	} catch {
		return branches;
	}

	const match = REF_REGEX.exec(head);
	if (match?.groups?.branch) branches.unshift(match.groups.branch);

	return branches;
}
