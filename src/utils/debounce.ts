/**
 * 防抖函数 - 延迟执行函数，直到用户停止操作指定时间后才执行
 * @param fn 要执行的函数
 * @param delay 延迟时间（毫秒）
 * @returns 防抖包装后的函数
 */
export const debounce = <T extends (...args: any[]) => any>(fn: T, delay: number) => {
  let timer: number | undefined;
  return (...args: Parameters<T>) => {
    if (timer) clearTimeout(timer);
    timer = setTimeout(() => {
      fn(...args);
      timer = undefined;
    }, delay) as unknown as number;
  };
};
