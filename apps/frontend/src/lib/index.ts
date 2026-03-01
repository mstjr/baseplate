// place files you want to import through the `$lib` alias in this folder.
export interface FunctionOutput {
	output: object;
	logs: Array<[string, { type: string; message: string }]>;
}
