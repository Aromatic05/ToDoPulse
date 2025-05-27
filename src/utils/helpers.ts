/**
 * 创建一个防抖函数
 * @param func 要防抖的函数
 * @param wait 等待时间（毫秒）
 * @returns 防抖后的函数
 */
export function debounce<T extends (...args: any[]) => any>(
	func: T,
	wait: number,
): (...args: Parameters<T>) => void {
	let timeout: ReturnType<typeof setTimeout> | null = null;

	return (...args: Parameters<T>): void => {
		const later = () => {
			timeout = null;
			func(...args);
		};

		if (timeout !== null) {
			clearTimeout(timeout);
		}
		timeout = setTimeout(later, wait);
	};
}
