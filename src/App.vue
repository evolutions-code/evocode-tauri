<template>
  <a-config-provider
    :theme="{
      algorithm: isDark ? theme.darkAlgorithm : theme.defaultAlgorithm,
      token: {
        colorPrimary: '#4d7dff',
        colorInfo: '#4d7dff',
        colorSuccess: '#34d399',
        colorWarning: '#fbbf24',
        colorError: '#f87171',
        colorBgBase: isDark ? '#0b0d12' : '#f5f7fb',
        colorTextBase: isDark ? '#f5f7fb' : '#0f172a',
        borderRadius: 10,
        fontFamily: '-apple-system, BlinkMacSystemFont, Segoe UI, Roboto, Helvetica Neue, PingFang SC, Microsoft YaHei, sans-serif',
        controlHeight: 36,
        wireframe: false,
      },
      components: {
        Layout: {
          headerBg: 'transparent',
          siderBg: 'transparent',
          bodyBg: 'transparent',
        },
        Menu: {
          itemBg: 'transparent',
          subMenuItemBg: 'transparent',
          itemSelectedBg: 'rgba(77,125,255,0.16)',
          itemSelectedColor: '#f5f7fb',
          itemHoverBg: 'rgba(255,255,255,0.04)',
        },
        Card: {
          colorBgContainer: 'var(--bg-elev-2)',
        },
        Tabs: {
          itemColor: 'var(--text-3)',
          itemHoverColor: 'var(--text-1)',
          itemSelectedColor: 'var(--text-1)',
        },
        Button: {
          colorPrimaryHover: '#6a96ff',
        },
      },
    }"
  >
    <a-app>
      <DefaultLayout>
        <router-view v-slot="{ Component, route }">
          <transition name="page" mode="out-in">
            <component :is="Component" :key="route.fullPath" />
          </transition>
        </router-view>
      </DefaultLayout>
    </a-app>
  </a-config-provider>
</template>

<script setup lang="ts">
import { computed } from 'vue'
import { theme } from 'ant-design-vue'
import DefaultLayout from './layouts/DefaultLayout.vue'
import { useTheme } from './composables/useTheme'

const { theme: themeMode } = useTheme()
const isDark = computed(() => themeMode.value === 'dark')
</script>

<style>
/* global tokens live in src/styles/global.css */
</style>
