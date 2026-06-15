<template>
  <a-config-provider
    :theme="{
      algorithm: isDark ? theme.darkAlgorithm : theme.defaultAlgorithm,
      token: {
        colorPrimary: '#4f7cff',
        colorInfo: '#4f7cff',
        colorSuccess: '#34d399',
        colorWarning: '#fbbf24',
        colorError: '#f87171',
        colorBgBase: isDark ? '#000000' : '#f5f5f5',
        colorTextBase: isDark ? '#ffffff' : '#111111',
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
          itemSelectedBg: 'rgba(255,255,255,0.10)',
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
          colorPrimaryHover: '#6b96ff',
          defaultBg: 'transparent',
          defaultColor: '#4f7cff',
          defaultBorderColor: '#4f7cff',
          defaultHoverBg: 'rgba(79,124,255,0.08)',
          defaultHoverColor: '#6b96ff',
          defaultHoverBorderColor: '#6b96ff',
          dashedBg: 'transparent',
          dashedColor: '#4f7cff',
          dashedBorderColor: '#4f7cff',
          dashedHoverBg: 'rgba(79,124,255,0.08)',
          dashedHoverColor: '#6b96ff',
          dashedHoverBorderColor: '#6b96ff',
        },
        Input: {
          colorBgContainer: 'rgba(255,255,255,0.04)',
          colorBorder: 'rgba(255,255,255,0.12)',
          colorPrimaryHover: '#4f7cff',
          colorPrimaryActive: '#4f7cff',
        },
        Select: {
          colorBgContainer: 'rgba(255,255,255,0.04)',
          colorBorder: 'rgba(255,255,255,0.12)',
          colorPrimaryHover: '#4f7cff',
          colorPrimaryActive: '#4f7cff',
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
