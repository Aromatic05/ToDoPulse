/**
 * 将字符串时间戳转换为日期格式 (MM-DD)
 * @param timestamp 时间戳字符串或数字
 * @returns 格式化的日期字符串
 */
export function convertTimestampToDate(timestamp: string): string {
  const date = new Date(Number(timestamp));
  
  // 获取年、月、日
//   const year = date.getFullYear();
  // getMonth() 返回 0-11，需要加1
  const month = String(date.getMonth() + 1).padStart(2, '0');
  const day = String(date.getDate()).padStart(2, '0');
  
  return `${month}-${day}`;
}

/**
 * 将字符串时间戳转换为时间格式 (HH:MM)
 * @param timestamp 时间戳字符串或数字
 * @returns 格式化的时间字符串
 */
export function convertTimestampToTime(timestamp: string): string {
  const date = new Date(Number(timestamp));
  
  // 获取时、分、秒
  const hours = String(date.getHours()).padStart(2, '0');
  const minutes = String(date.getMinutes()).padStart(2, '0');
//   const seconds = String(date.getSeconds()).padStart(2, '0');
  
  return `${hours}:${minutes}`;
}