import en from 'element-plus/es/locale/lang/en'
import zhCn from 'element-plus/es/locale/lang/zh-cn'

export const locales = {
  'zh-cn': zhCn,
  'en': en,
}

export type LocaleKey = keyof typeof locales

export const defaultLocale: LocaleKey = 'zh-cn'
