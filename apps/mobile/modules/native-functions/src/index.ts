import NativeFunctionsModule from './NativeFunctionsModule';

export function getTheme(): string {
	return NativeFunctionsModule.getTheme();
}

export function hello(): string {
	return NativeFunctionsModule.hello();
}
