export class ProxyState<T> {
	value: T = $state<T>()!;

	constructor(initialValue: T) {
		this.value = initialValue;
	}
}
