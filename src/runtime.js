((globalThis) => {
	const core = Deno.core;

	function argsToMessage(...args) {
		return args.map((arg) => JSON.stringify(arg)).join(" ");
	}

	globalThis.console = {
		log: (...args) => {
			core.print(`${argsToMessage(...args)}`, false);
		},
		error: (...args) => {
			core.print(`[err]: ${argsToMessage(...args)}\n`, true);
		},
	};
})(globalThis);
